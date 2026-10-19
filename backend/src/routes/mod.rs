pub mod admissions;
pub mod auth;
pub mod batches;
pub mod dashboards;
pub mod institutions;
pub mod issues;
pub mod reports;
pub mod verifications;

use axum::routing::get;
use axum::Router;
use serde::Serialize;

use crate::error::{AppError, AppResult};
use crate::repositories;
use crate::state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub version: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<&'static str>,
}

/// Liveness probe. Never exposes internal detail.
pub async fn health_live() -> AppResult<Json<HealthResponse>> {
    Ok(Json(HealthResponse {
        status: "ok",
        service: "aicte-apaar-api",
        version: env!("CARGO_PKG_VERSION"),
        database: None,
    }))
}

/// Readiness probe. Reports whether the database is reachable, nothing more.
pub async fn health_ready(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> AppResult<Json<HealthResponse>> {
    repositories::ping(&state.pool)
        .await
        .map_err(|_| AppError::internal(anyhow::anyhow!("database not ready")))?;
    Ok(Json(HealthResponse {
        status: "ok",
        service: "aicte-apaar-api",
        version: env!("CARGO_PKG_VERSION"),
        database: Some("up"),
    }))
}

use axum::Json;

/// Assembles the complete application router.
pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/auth/login", axum::routing::post(auth::login))
        .route("/me", get(auth::me))
        .route("/institutions", get(institutions::list_institutions))
        .route(
            "/institutions/{id}/intakes",
            get(institutions::list_intakes),
        )
        .route(
            "/institutions/{id}/programmes",
            get(institutions::list_programmes),
        )
        .route("/template", get(institutions::download_template))
        .route("/batches", axum::routing::post(batches::create_batch))
        .route("/batches", get(batches::list_batches))
        .route("/batches/{id}", get(batches::get_batch))
        .route("/batches/{id}/issues", get(batches::list_batch_issues))
        .route(
            "/batches/{id}/commit",
            axum::routing::post(batches::commit_batch),
        )
        .route("/admissions", get(admissions::search_admissions))
        .route("/issues", get(issues::list_open_issues))
        .route(
            "/issues/{id}/resolve",
            axum::routing::post(issues::resolve_issue),
        )
        .route(
            "/dashboard/institution",
            get(dashboards::institution_dashboard),
        )
        .route("/dashboard/aicte", get(dashboards::aicte_dashboard))
        .route("/verifications", axum::routing::post(verifications::verify))
        .route("/reports/enrolment", get(reports::enrolment_report))
        .route("/audit", get(reports::audit_log));

    Router::new()
        .route("/health/live", get(health_live))
        .route("/health/ready", get(health_ready))
        .nest("/api/v1", api)
        .with_state(state)
}
