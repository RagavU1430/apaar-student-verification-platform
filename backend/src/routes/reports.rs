use axum::extract::{Query, State};
use axum::http::header;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::auth::policy::AuthenticatedUser;
use crate::error::{AppError, AppResult};
use crate::repositories::admissions;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ReportQuery {
    pub institution_id: Option<String>,
    pub academic_year: Option<String>,
    pub programme_code: Option<String>,
    pub format: Option<String>,
}

/// Authorised enrolment report.
///
/// The export is filtered by the caller's scope and is volume-capped. APAAR
/// ids are masked in the output.
#[derive(Serialize)]
pub struct EnrolmentRow {
    pub institution_id: String,
    pub programme_code: String,
    pub academic_year: String,
    pub seat_category: String,
    pub status: String,
    pub count: i64,
}

/// Returns the enrolment report as JSON or CSV.
pub async fn enrolment_report(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<ReportQuery>,
) -> AppResult<impl IntoResponse> {
    let scope = match query.institution_id.as_deref() {
        Some(raw) => {
            let id = uuid::Uuid::parse_str(raw)
                .map_err(|_| AppError::bad_request("institution_id must be a UUID."))?;
            user.require_institution_scope(id)?;
            crate::auth::policy::ScopeFilter::Institution(id)
        }
        None => user.scope_filter(),
    };

    let filters = admissions::AdmissionFilters {
        programme_code: query.programme_code,
        academic_year: query.academic_year,
        ..Default::default()
    };

    let (rows, total) = admissions::search(&state.pool, scope, &filters, 500, 0).await?;

    let mut grouped: Vec<(String, String, String, String, i64)> = Vec::new();
    for row in &rows {
        let key = (
            row.institution_id.to_string(),
            row.programme_code.clone(),
            row.academic_year.clone(),
            row.seat_category.clone(),
        );
        match grouped
            .iter_mut()
            .find(|g| (g.0.clone(), g.1.clone(), g.2.clone(), g.3.clone()) == key)
        {
            Some(entry) => entry.4 += 1,
            None => grouped.push((key.0, key.1, key.2, key.3, 1)),
        }
    }

    let items: Vec<EnrolmentRow> = grouped
        .into_iter()
        .map(
            |(institution_id, programme_code, academic_year, seat_category, count)| EnrolmentRow {
                institution_id,
                programme_code,
                academic_year,
                seat_category,
                status: "ACTIVE".to_string(),
                count,
            },
        )
        .collect();

    if query.format.as_deref() == Some("csv") {
        let mut csv = String::from(
            "institution_id,programme_code,academic_year,seat_category,status,count\n",
        );
        for item in &items {
            csv.push_str(&format!(
                "{},{},{},{},{},{}\n",
                item.institution_id,
                item.programme_code,
                item.academic_year,
                item.seat_category,
                item.status,
                item.count
            ));
        }
        return Ok((
            [
                (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"enrolment-report.csv\"",
                ),
            ],
            csv,
        )
            .into_response());
    }

    Ok(Json(serde_json::json!({
        "total_matching_records": total,
        "rows_returned": items.len(),
        "rule_version": state.config.demo_rule_version,
        "items": items,
    }))
    .into_response())
}

/// The audit trail, readable by regulators and auditors.
#[derive(Serialize)]
pub struct AuditEventView {
    pub id: String,
    pub actor_id: Option<String>,
    pub actor_role: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub institution_id: Option<String>,
    pub outcome: String,
    pub summary: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct AuditRow {
    id: uuid::Uuid,
    actor_id: Option<uuid::Uuid>,
    actor_role: Option<String>,
    action: String,
    entity_type: String,
    entity_id: Option<uuid::Uuid>,
    institution_id: Option<uuid::Uuid>,
    outcome: String,
    summary: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn audit_log(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<Vec<AuditEventView>>> {
    if !user.role.can_view_all_institutions() && !user.role.can_review() {
        return Err(AppError::forbidden(
            "Your role cannot read the audit trail.",
        ));
    }

    let rows: Vec<AuditRow> = sqlx::query_as::<_, AuditRow>(
        "SELECT id, actor_id, actor_role, action, entity_type, entity_id, institution_id,
                outcome, summary, created_at
         FROM audit_events ORDER BY created_at DESC LIMIT 200",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::internal)?;

    Ok(Json(
        rows.into_iter()
            .map(|row| AuditEventView {
                id: row.id.to_string(),
                actor_id: row.actor_id.map(|a| a.to_string()),
                actor_role: row.actor_role,
                action: row.action,
                entity_type: row.entity_type,
                entity_id: row.entity_id.map(|e| e.to_string()),
                institution_id: row.institution_id.map(|i| i.to_string()),
                outcome: row.outcome,
                summary: row.summary,
                created_at: row.created_at,
            })
            .collect(),
    ))
}
