use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

use crate::auth::policy::AuthenticatedUser;
use crate::error::{AppError, AppResult};
use crate::repositories::{batches, issues, staging};
use crate::services::audit;
use crate::services::ingestion::{self, SCHEMA_VERSION};
use crate::services::intake_monitoring;
use crate::services::validation;
use crate::state::AppState;

#[derive(Serialize)]
pub struct BatchSummary {
    pub id: String,
    pub institution_id: String,
    pub file_name: String,
    pub schema_version: String,
    pub status: String,
    pub counts: BatchCountsView,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
    pub committed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Rule version that produced the findings.
    pub rule_version: String,
    /// Always true in the prototype: there is no live APAAR check.
    pub external_verification: &'static str,
}

#[derive(Serialize, Clone, Copy)]
pub struct BatchCountsView {
    pub rows: i32,
    pub accepted: i32,
    pub accepted_with_warning: i32,
    pub needs_review: i32,
    pub duplicates: i32,
    pub rejected: i32,
}

impl From<&batches::BatchRow> for BatchSummary {
    fn from(b: &batches::BatchRow) -> Self {
        BatchSummary {
            id: b.id.to_string(),
            institution_id: b.institution_id.to_string(),
            file_name: b.file_name.clone(),
            schema_version: b.schema_version.clone(),
            status: b.status.clone(),
            counts: BatchCountsView {
                rows: b.row_count,
                accepted: b.accepted_count,
                accepted_with_warning: b.warning_count,
                needs_review: b.needs_review_count,
                duplicates: b.duplicate_count,
                rejected: b.rejected_count,
            },
            submitted_at: b.submitted_at,
            committed_at: b.committed_at,
            rule_version: crate::config::Config::from_env()
                .map(|c| c.demo_rule_version)
                .unwrap_or_else(|_| "demo-v1".to_string()),
            external_verification: "not performed - format checks only",
        }
    }
}

#[derive(Serialize)]
pub struct IssueView {
    pub id: String,
    pub row_reference: String,
    pub rule_code: String,
    pub severity: String,
    pub message: String,
    pub field: Option<String>,
    pub evidence: Option<String>,
    pub rule_version: String,
    pub status: String,
    pub resolution_note: Option<String>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Uploads a CSV batch and validates every row.
///
/// File level problems fail the whole request. Row level problems are reported
/// per row; no row is ever silently discarded.
pub async fn create_batch(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    mut multipart: Multipart,
) -> AppResult<(StatusCode, Json<BatchSummary>)> {
    if !user.role.can_upload() {
        audit::record_denied(
            &state.pool,
            &user,
            "batch.create",
            "submission_batch",
            None,
            user.institution_id,
            "role is not allowed to upload batches",
        )
        .await?;
        return Err(AppError::forbidden("Your role cannot upload batches."));
    }

    let institution_id = user
        .institution_id
        .ok_or_else(|| AppError::forbidden("This action requires an institution-scoped account."))?;

    let mut file_name = "upload.csv".to_string();
    let mut file_bytes: Vec<u8> = Vec::new();
    let mut idempotency_key: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("Could not read the upload: {e}")))?
    {
        match field.name() {
            Some("file") => {
                file_name = field
                    .file_name()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "upload.csv".to_string());
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::bad_request(format!("Could not read the file: {e}")))?;
                if data.len() > state.config.max_upload_bytes {
                    return Err(AppError::PayloadTooLarge(format!(
                        "The file is larger than the {} byte limit.",
                        state.config.max_upload_bytes
                    )));
                }
                file_bytes = data.to_vec();
            }
            Some("idempotency_key") => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::bad_request(format!("Invalid idempotency key: {e}")))?;
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    idempotency_key = Some(trimmed.to_string());
                }
            }
            _ => {}
        }
    }

    if file_bytes.is_empty() {
        return Err(AppError::bad_request(
            "No file was uploaded. Send the CSV as the 'file' field.",
        ));
    }

    if !file_name.to_ascii_lowercase().ends_with(".csv") {
        return Err(AppError::row(
            "FILE_UNSUPPORTED_TYPE",
            "Only .csv files are accepted.",
        ));
    }

    // A repeated idempotency key returns the existing batch instead of
    // creating a second one.
    if let Some(key) = idempotency_key.as_deref() {
        if let Some(existing) = batches::find_by_idempotency_key(&state.pool, key).await? {
            if existing.institution_id == institution_id {
                return Ok((StatusCode::OK, Json(BatchSummary::from(&existing))));
            }
        }
    }

    let rows = ingestion::parse_csv(&file_bytes, state.config.max_batch_rows)?;
    let hash = ingestion::file_hash(&file_bytes);

    let validation = validation::validate_rows(&state.pool, institution_id, &rows, &state.config).await?;

    let batch_id = batches::create(
        &state.pool,
        institution_id,
        user.id,
        &file_name,
        &hash,
        SCHEMA_VERSION,
        idempotency_key.as_deref(),
    )
    .await?;

    // Persist the findings and the accepted rows for the commit step.
    for outcome in &validation.outcomes {
        for finding in &outcome.findings {
            issues::insert(
                &state.pool,
                batch_id,
                institution_id,
                &format!("row {}", outcome.row_number),
                finding.rule_code.as_str(),
                finding.severity.as_str(),
                &finding.message,
                finding.field.as_deref(),
                finding.evidence.as_deref(),
                &finding.rule_version,
            )
            .await?;
        }

        if let (Some(candidate), Some(programme_id)) = (&outcome.candidate, outcome.programme_id) {
            if matches!(
                outcome.status,
                crate::domain::validation::ValidationStatus::Accepted
                    | crate::domain::validation::ValidationStatus::AcceptedWithWarning
            ) {
                staging::insert(
                    &state.pool,
                    batch_id,
                    outcome.row_number as i32,
                    &candidate.apaar_id,
                    &candidate.institution_student_id,
                    candidate.full_name.as_deref(),
                    programme_id,
                    &candidate.academic_year,
                    candidate.admission_date,
                    candidate.status.as_str(),
                    candidate.seat_category.as_str(),
                    outcome.status.as_str(),
                )
                .await?;
            }
        }
    }

    batches::update_counts(&state.pool, batch_id, &validation.counts).await?;

    audit::record(
        &state.pool,
        Some(&user),
        "batch.create",
        "submission_batch",
        Some(batch_id),
        Some(institution_id),
        "SUCCESS",
        None,
        Some(format!(
            "uploaded {} rows: {} accepted, {} rejected, {} need review",
            validation.counts.rows,
            validation.counts.accepted,
            validation.counts.rejected,
            validation.counts.needs_review
        )),
    )
    .await?;

    let row = batches::get(&state.pool, batch_id)
        .await?
        .ok_or_else(|| AppError::internal(anyhow::anyhow!("batch vanished")))?;

    Ok((StatusCode::CREATED, Json(BatchSummary::from(&row))))
}

/// Lists the batches visible to the caller.
pub async fn list_batches(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<Vec<BatchSummary>>> {
    let rows = batches::list(&state.pool, user.scope_filter()).await?;
    Ok(Json(rows.iter().map(BatchSummary::from).collect()))
}

/// Returns the status and aggregate results of one batch.
pub async fn get_batch(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<BatchSummary>> {
    let row = batches::get(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::not_found("Batch not found."))?;

    if let Err(err) = user.require_institution_scope(row.institution_id) {
        audit::record_denied(
            &state.pool,
            &user,
            "batch.read",
            "submission_batch",
            Some(id),
            Some(row.institution_id),
            "cross-institution batch read denied",
        )
        .await?;
        return Err(err);
    }

    Ok(Json(BatchSummary::from(&row)))
}

/// Returns the row level validation issues of one batch.
pub async fn list_batch_issues(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<IssueView>>> {
    let row = batches::get(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::not_found("Batch not found."))?;

    if let Err(err) = user.require_institution_scope(row.institution_id) {
        audit::record_denied(
            &state.pool,
            &user,
            "batch.issues.read",
            "submission_batch",
            Some(id),
            Some(row.institution_id),
            "cross-institution issue read denied",
        )
        .await?;
        return Err(err);
    }

    let rows = issues::list_for_batch(&state.pool, id).await?;
    Ok(Json(rows.iter().map(issue_view).collect()))
}

fn issue_view(row: &issues::IssueRow) -> IssueView {
    IssueView {
        id: row.id.to_string(),
        row_reference: row.row_reference.clone(),
        rule_code: row.rule_code.clone(),
        severity: row.severity.clone(),
        message: row.message.clone(),
        field: row.field.clone(),
        evidence: row.evidence.clone(),
        rule_version: row.rule_version.clone(),
        status: row.status.clone(),
        resolution_note: row.resolution_note.clone(),
        resolved_at: row.resolved_at,
    }
}

#[derive(Serialize)]
pub struct CommitResponse {
    pub batch_id: String,
    pub status: String,
    pub committed: i64,
    pub skipped: i64,
    pub intake_exceptions: Vec<IntakeExceptionView>,
}

/// Commits the accepted rows of a batch and recomputes intake.
///
/// The whole commit runs in one transaction: a failure leaves no partial
/// state.
pub async fn commit_batch(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    if !user.role.can_commit() {
        audit::record_denied(
            &state.pool,
            &user,
            "batch.commit",
            "submission_batch",
            Some(id),
            user.institution_id,
            "role is not allowed to commit batches",
        )
        .await?;
        return Err(AppError::forbidden("Your role cannot commit batches."));
    }

    let row = batches::get(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::not_found("Batch not found."))?;

    if let Err(err) = user.require_institution_scope(row.institution_id) {
        audit::record_denied(
            &state.pool,
            &user,
            "batch.commit",
            "submission_batch",
            Some(id),
            Some(row.institution_id),
            "cross-institution commit denied",
        )
        .await?;
        return Err(err);
    }

    let summary = ingestion::commit_batch(
        &state.pool,
        id,
        row.institution_id,
        user.id,
        &state.config.demo_rule_version,
    )
    .await?;

    // Recompute intake and raise an exception when the capacity is exceeded.
    let usages = intake_monitoring::monitor(&state.pool, &state.config, Some(row.institution_id)).await?;
    let mut exceptions = Vec::new();
    for usage in usages {
        if usage.over_capacity || usage.supernumerary_over_capacity {
            let message = if usage.over_capacity {
                format!(
                    "Active regular admissions ({}) exceed the configured regular intake ({}). Excess: {}.",
                    usage.regular_count, usage.regular_intake, usage.excess_over_regular
                )
            } else {
                format!(
                    "Active supernumerary admissions ({}) exceed the configured supernumerary capacity ({}). Excess: {}.",
                    usage.supernumerary_count, usage.supernumerary_capacity, usage.excess_over_supernumerary
                )
            };

            issues::insert(
                &state.pool,
                id,
                row.institution_id,
                &format!("{} / {}", usage.programme_code, usage.academic_year),
                "INTAKE_CAPACITY_EXCEEDED",
                "REVIEW",
                &message,
                None,
                Some(&format!(
                    "rule {} | counted {} regular / {} supernumerary",
                    usage.rule_version, usage.regular_count, usage.supernumerary_count
                )),
                &usage.rule_version,
            )
            .await?;

            exceptions.push(IntakeExceptionView {
                programme_code: usage.programme_code.clone(),
                academic_year: usage.academic_year.clone(),
                rule_version: usage.rule_version.clone(),
                message: message.clone(),
                regular_count: usage.regular_count,
                regular_intake: usage.regular_intake,
                excess: usage.excess_over_regular,
            });
        }
    }

    Ok((
        StatusCode::OK,
        Json(CommitResponse {
            batch_id: id.to_string(),
            status: "COMMITTED".to_string(),
            committed: summary.committed,
            skipped: summary.skipped,
            intake_exceptions: exceptions,
        }),
    ))
}

#[derive(Serialize)]
pub struct IntakeExceptionView {
    pub programme_code: String,
    pub academic_year: String,
    pub rule_version: String,
    pub message: String,
    pub regular_count: i64,
    pub regular_intake: i64,
    pub excess: i64,
}
