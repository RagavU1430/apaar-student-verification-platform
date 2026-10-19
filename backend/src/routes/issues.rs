use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::policy::AuthenticatedUser;
use crate::error::{AppError, AppResult};
use crate::repositories::issues;
use crate::services::audit;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct IssueQuery {
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct IssueView {
    pub id: String,
    pub batch_id: String,
    pub institution_id: String,
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

/// Lists the open issues in the caller's scope, most severe first.
pub async fn list_open_issues(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<IssueQuery>,
) -> AppResult<Json<Vec<IssueView>>> {
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let rows = issues::list_open(&state.pool, user.scope_filter(), limit).await?;
    Ok(Json(rows.iter().map(view).collect()))
}

fn view(row: &issues::IssueRow) -> IssueView {
    IssueView {
        id: row.id.to_string(),
        batch_id: row.batch_id.to_string(),
        institution_id: row.institution_id.to_string(),
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

#[derive(Deserialize)]
pub struct ResolveRequest {
    /// `RESOLVED` or `DISMISSED`.
    pub status: String,
    pub note: String,
}

/// Records a review outcome for an issue.
///
/// The original finding and its evidence are never deleted: the outcome is
/// appended to the history.
pub async fn resolve_issue(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    axum::Json(request): axum::Json<ResolveRequest>,
) -> AppResult<Json<IssueView>> {
    if !user.role.can_review() {
        audit::record_denied(
            &state.pool,
            &user,
            "issue.resolve",
            "validation_issue",
            Some(id),
            user.institution_id,
            "role is not allowed to review issues",
        )
        .await?;
        return Err(AppError::forbidden("Your role cannot review issues."));
    }

    let issue = issues::get(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::not_found("Issue not found."))?;

    if let Err(err) = user.require_institution_scope(issue.institution_id) {
        audit::record_denied(
            &state.pool,
            &user,
            "issue.resolve",
            "validation_issue",
            Some(id),
            Some(issue.institution_id),
            "cross-institution issue review denied",
        )
        .await?;
        return Err(err);
    }

    let status = request.status.trim().to_ascii_uppercase();
    if !matches!(status.as_str(), "RESOLVED" | "DISMISSED") {
        return Err(AppError::bad_request(
            "Status must be RESOLVED or DISMISSED.",
        ));
    }
    if request.note.trim().is_empty() {
        return Err(AppError::bad_request(
            "A reason is required to close an issue.",
        ));
    }

    issues::resolve(&state.pool, id, &status, request.note.trim(), user.id).await?;

    audit::record(
        &state.pool,
        Some(&user),
        "issue.resolve",
        "validation_issue",
        Some(id),
        Some(issue.institution_id),
        "SUCCESS",
        None,
        Some(format!("{status}: {}", request.note.trim())),
    )
    .await?;

    let updated = issues::get(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::internal(anyhow::anyhow!("issue vanished")))?;

    Ok(Json(view(&updated)))
}
