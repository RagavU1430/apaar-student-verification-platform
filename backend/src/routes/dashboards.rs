use axum::extract::State;
use axum::Json;
use serde::Serialize;

use crate::auth::policy::AuthenticatedUser;
use crate::error::{AppError, AppResult};
use crate::repositories::{batches, issues};
use crate::services::intake_monitoring;
use crate::state::AppState;

#[derive(Serialize)]
pub struct InstitutionDashboard {
    pub institution_id: String,
    pub institution_name: Option<String>,
    pub rule_version: String,
    pub data_source: &'static str,
    pub batches: Vec<BatchCard>,
    pub intake: Vec<IntakeCard>,
    pub open_issues: i64,
    pub counts: DashboardCounts,
}

#[derive(Serialize)]
pub struct DashboardCounts {
    pub total_batches: i64,
    pub total_rows: i64,
    pub accepted_rows: i64,
    pub rejected_rows: i64,
    pub rows_needing_review: i64,
    pub duplicate_candidates: i64,
}

#[derive(Serialize)]
pub struct BatchCard {
    pub id: String,
    pub file_name: String,
    pub status: String,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
    pub rows: i32,
    pub accepted: i32,
    pub rejected: i32,
    pub needs_review: i32,
}

#[derive(Serialize)]
pub struct IntakeCard {
    pub programme_code: String,
    pub programme_name: Option<String>,
    pub academic_year: String,
    pub rule_version: String,
    pub regular_count: i64,
    pub regular_intake: i64,
    pub supernumerary_count: i64,
    pub supernumerary_capacity: i64,
    pub utilisation: Option<f64>,
    pub over_capacity: bool,
    pub source_reference: Option<String>,
}

/// The institution dashboard: own records, own issues, own intake utilisation.
pub async fn institution_dashboard(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<InstitutionDashboard>> {
    let institution_id = user
        .institution_id
        .ok_or_else(|| AppError::forbidden("This dashboard requires an institution-scoped account."))?;

    let institution = crate::repositories::institutions::get(&state.pool, institution_id).await?;
    let rows = batches::list(&state.pool, crate::auth::policy::ScopeFilter::Institution(institution_id)).await?;
    let usages = intake_monitoring::monitor(&state.pool, &state.config, Some(institution_id)).await?;
    let open = issues::list_open(
        &state.pool,
        crate::auth::policy::ScopeFilter::Institution(institution_id),
        500,
    )
    .await?;

    let counts = DashboardCounts {
        total_batches: rows.len() as i64,
        total_rows: rows.iter().map(|r| r.row_count).sum::<i32>() as i64,
        accepted_rows: rows.iter().map(|r| r.accepted_count).sum::<i32>() as i64,
        rejected_rows: rows.iter().map(|r| r.rejected_count).sum::<i32>() as i64,
        rows_needing_review: rows.iter().map(|r| r.needs_review_count).sum::<i32>() as i64,
        duplicate_candidates: rows.iter().map(|r| r.duplicate_count).sum::<i32>() as i64,
    };

    let intakes = crate::repositories::institutions::list_intakes(&state.pool, institution_id).await?;
    let source_by_programme: Vec<(String, String, String)> = intakes
        .iter()
        .map(|i| {
            (
                i.programme_id.to_string(),
                i.programme_name.clone(),
                i.source_reference.clone(),
            )
        })
        .collect();

    let cards = usages
        .into_iter()
        .map(|u| {
            let meta = source_by_programme
                .iter()
                .find(|(pid, _, _)| *pid == u.programme_id);
            IntakeCard {
                programme_code: u.programme_code,
                programme_name: meta.map(|(_, name, _)| name.clone()),
                academic_year: u.academic_year,
                rule_version: u.rule_version,
                regular_count: u.regular_count,
                regular_intake: u.regular_intake,
                supernumerary_count: u.supernumerary_count,
                supernumerary_capacity: u.supernumerary_capacity,
                utilisation: crate::domain::intake::intake_utilisation(
                    u.regular_count,
                    u.regular_intake,
                ),
                over_capacity: u.over_capacity || u.supernumerary_over_capacity,
                source_reference: meta.map(|(_, _, src)| src.clone()),
            }
        })
        .collect();

    Ok(Json(InstitutionDashboard {
        institution_id: institution_id.to_string(),
        institution_name: institution.map(|i| i.name),
        rule_version: state.config.demo_rule_version.clone(),
        data_source: "platform records (demo)",
        batches: rows
            .iter()
            .map(|r| BatchCard {
                id: r.id.to_string(),
                file_name: r.file_name.clone(),
                status: r.status.clone(),
                submitted_at: r.submitted_at,
                rows: r.row_count,
                accepted: r.accepted_count,
                rejected: r.rejected_count,
                needs_review: r.needs_review_count,
            })
            .collect(),
        intake: cards,
        open_issues: open.len() as i64,
        counts,
    }))
}

#[derive(Serialize)]
pub struct AicteDashboard {
    pub rule_version: String,
    pub data_source: &'static str,
    pub institutions: Vec<InstitutionSummary>,
    pub intake: Vec<IntakeCard>,
    pub exception_queue: Vec<ExceptionCard>,
    pub counts: DashboardCounts,
}

#[derive(Serialize)]
pub struct InstitutionSummary {
    pub id: String,
    pub name: String,
    pub official_code: String,
    pub active_admissions: i64,
}

#[derive(Serialize)]
pub struct ExceptionCard {
    pub issue_id: String,
    pub institution_id: String,
    pub row_reference: String,
    pub rule_code: String,
    pub message: String,
    pub evidence: Option<String>,
    pub rule_version: String,
    pub severity: String,
}

/// The AICTE dashboard: cross-institution aggregates and the review queue.
pub async fn aicte_dashboard(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<AicteDashboard>> {
    if !user.role.can_view_all_institutions() {
        return Err(AppError::forbidden(
            "Only the AICTE role can view cross-institution aggregates.",
        ));
    }

    let institutions = crate::repositories::institutions::list_visible(
        &state.pool,
        crate::auth::policy::ScopeFilter::All,
    )
    .await?;
    let usages = intake_monitoring::monitor(&state.pool, &state.config, None).await?;
    let open = issues::list_open(&state.pool, crate::auth::policy::ScopeFilter::All, 100).await?;
    let batches = batches::list(&state.pool, crate::auth::policy::ScopeFilter::All).await?;

    let counts = DashboardCounts {
        total_batches: batches.len() as i64,
        total_rows: batches.iter().map(|r| r.row_count).sum::<i32>() as i64,
        accepted_rows: batches.iter().map(|r| r.accepted_count).sum::<i32>() as i64,
        rejected_rows: batches.iter().map(|r| r.rejected_count).sum::<i32>() as i64,
        rows_needing_review: batches.iter().map(|r| r.needs_review_count).sum::<i32>() as i64,
        duplicate_candidates: batches.iter().map(|r| r.duplicate_count).sum::<i32>() as i64,
    };

    let mut summaries = Vec::new();
    for institution in institutions {
        let active: i64 = usages
            .iter()
            .filter(|u| u.institution_id == institution.id.to_string())
            .map(|u| u.active_count)
            .sum();
        summaries.push(InstitutionSummary {
            id: institution.id.to_string(),
            name: institution.name,
            official_code: institution.official_code,
            active_admissions: active,
        });
    }

    Ok(Json(AicteDashboard {
        rule_version: state.config.demo_rule_version.clone(),
        data_source: "platform records (demo)",
        institutions: summaries,
        intake: usages
            .into_iter()
            .map(|u| IntakeCard {
                programme_code: u.programme_code,
                programme_name: None,
                academic_year: u.academic_year,
                rule_version: u.rule_version,
                regular_count: u.regular_count,
                regular_intake: u.regular_intake,
                supernumerary_count: u.supernumerary_count,
                supernumerary_capacity: u.supernumerary_capacity,
                utilisation: crate::domain::intake::intake_utilisation(
                    u.regular_count,
                    u.regular_intake,
                ),
                over_capacity: u.over_capacity || u.supernumerary_over_capacity,
                source_reference: None,
            })
            .collect(),
        exception_queue: open
            .iter()
            .filter(|i| i.severity == "REVIEW" || i.rule_code == "INTAKE_CAPACITY_EXCEEDED")
            .map(|i| ExceptionCard {
                issue_id: i.id.to_string(),
                institution_id: i.institution_id.to_string(),
                row_reference: i.row_reference.clone(),
                rule_code: i.rule_code.clone(),
                message: i.message.clone(),
                evidence: i.evidence.clone(),
                rule_version: i.rule_version.clone(),
                severity: i.severity.clone(),
            })
            .collect(),
        counts,
    }))
}
