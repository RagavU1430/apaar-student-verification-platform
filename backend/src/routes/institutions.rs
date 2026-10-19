use axum::extract::{Path, State};
use axum::http::header;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

use crate::auth::policy::AuthenticatedUser;
use crate::error::{AppError, AppResult};
use crate::repositories::institutions;
use crate::state::AppState;
use crate::services::audit;

#[derive(Serialize)]
pub struct InstitutionSummary {
    pub id: String,
    pub official_code: String,
    pub name: String,
    pub status: String,
}

/// Lists the institutions the caller may see.
pub async fn list_institutions(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<Vec<InstitutionSummary>>> {
    let visible = institutions::list_visible(&state.pool, user.scope_filter()).await?;
    Ok(Json(
        visible
            .into_iter()
            .map(|i| InstitutionSummary {
                id: i.id.to_string(),
                official_code: i.official_code,
                name: i.name,
                status: i.status,
            })
            .collect(),
    ))
}

#[derive(Serialize)]
pub struct IntakeSummary {
    pub programme_id: String,
    pub programme_code: String,
    pub programme_name: String,
    pub academic_year: String,
    pub regular_intake: i32,
    pub supernumerary_capacity: i32,
    pub source_reference: String,
    pub rule_version: String,
    pub effective_version: i32,
}

/// Reads the approved intake configuration for one institution.
pub async fn list_intakes(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<IntakeSummary>>> {
    if let Err(err) = user.require_institution_scope(id) {
        audit::record_denied(
            &state.pool,
            &user,
            "institution.intakes.read",
            "institution",
            Some(id),
            Some(id),
            "cross-institution intake read denied",
        )
        .await?;
        return Err(err);
    }

    let intakes = institutions::list_intakes(&state.pool, id).await?;
    Ok(Json(
        intakes
            .into_iter()
            .map(|i| IntakeSummary {
                programme_id: i.programme_id.to_string(),
                programme_code: i.programme_code,
                programme_name: i.programme_name,
                academic_year: i.academic_year,
                regular_intake: i.regular_intake,
                supernumerary_capacity: i.supernumerary_capacity,
                source_reference: i.source_reference,
                rule_version: i.rule_version,
                effective_version: i.effective_version,
            })
            .collect(),
    ))
}

#[derive(Serialize)]
pub struct ProgrammeSummary {
    pub id: String,
    pub code: String,
    pub name: String,
    pub level: String,
    pub status: String,
}

/// Lists the programmes of one institution.
pub async fn list_programmes(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<ProgrammeSummary>>> {
    if let Err(err) = user.require_institution_scope(id) {
        audit::record_denied(
            &state.pool,
            &user,
            "institution.programmes.read",
            "institution",
            Some(id),
            Some(id),
            "cross-institution programme read denied",
        )
        .await?;
        return Err(err);
    }

    let programmes = institutions::list_programmes(&state.pool, id).await?;
    Ok(Json(
        programmes
            .into_iter()
            .map(|p| ProgrammeSummary {
                id: p.id.to_string(),
                code: p.code,
                name: p.name,
                level: p.level,
                status: p.status,
            })
            .collect(),
    ))
}

/// Downloads the versioned CSV template.
pub async fn download_template() -> AppResult<impl IntoResponse> {
    let body = crate::services::ingestion::template_bytes();
    Ok((
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"apaar-admissions-template-v1.csv\"",
            ),
        ],
        body,
    ))
}

/// Helper used by the batch routes to resolve the institution of the caller.
pub fn caller_institution(user: &AuthenticatedUser) -> AppResult<Uuid> {
    user.institution_id
        .ok_or_else(|| AppError::forbidden("This action requires an institution-scoped account."))
}
