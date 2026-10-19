use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::auth::policy::AuthenticatedUser;
use crate::error::AppResult;
use crate::repositories::admissions::{self, AdmissionFilters};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct AdmissionQuery {
    pub programme_code: Option<String>,
    pub academic_year: Option<String>,
    pub status: Option<String>,
    pub seat_category: Option<String>,
    pub search: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Serialize)]
pub struct AdmissionRow {
    pub id: String,
    pub institution_id: String,
    pub programme_code: String,
    pub academic_year: String,
    pub admission_date: chrono::NaiveDate,
    pub status: String,
    pub seat_category: String,
    pub institution_student_id: String,
    pub full_name: Option<String>,
    /// APAAR ids are masked in list views.
    pub apaar_id_masked: String,
}

#[derive(Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// Searches admissions inside the caller's scope.
pub async fn search_admissions(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<AdmissionQuery>,
) -> AppResult<Json<Page<AdmissionRow>>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query
        .page_size
        .unwrap_or(state.config.default_page_size)
        .clamp(1, state.config.max_page_size);

    let filters = AdmissionFilters {
        programme_code: query.programme_code,
        academic_year: query.academic_year,
        status: query.status,
        seat_category: query.seat_category,
        search: query.search,
    };

    let (rows, total) = admissions::search(
        &state.pool,
        user.scope_filter(),
        &filters,
        page_size,
        (page - 1) * page_size,
    )
    .await?;

    let items = rows
        .into_iter()
        .map(|r| AdmissionRow {
            id: r.id.to_string(),
            institution_id: r.institution_id.to_string(),
            programme_code: r.programme_code,
            academic_year: r.academic_year,
            admission_date: r.admission_date,
            status: r.status,
            seat_category: r.seat_category,
            institution_student_id: r.institution_student_id,
            full_name: r.full_name,
            apaar_id_masked: r.apaar_id_masked,
        })
        .collect();

    Ok(Json(Page {
        items,
        total,
        page,
        page_size,
    }))
}
