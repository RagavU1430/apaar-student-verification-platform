//! Shared helpers for the integration tests.
//!
//! The integration tests need a PostgreSQL database. When `DATABASE_URL` is not
//! set they skip themselves instead of failing, so `cargo test` still works on
//! a machine without a database.

use std::sync::Arc;

use aicte_apaar_api::config::Config;
use aicte_apaar_api::state::AppState;
use sqlx::postgres::PgPoolOptions;

pub fn database_configured() -> bool {
    std::env::var("DATABASE_URL").is_ok()
}

/// Builds the application state against the configured database.
pub async fn test_state() -> Option<AppState> {
    if !database_configured() {
        return None;
    }
    let config = Config::from_env().ok()?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .ok()?;
    sqlx::migrate!("./migrations").run(&pool).await.ok()?;
    Some(AppState {
        pool,
        config: Arc::new(config),
    })
}

/// Signs in and returns the access token.
pub async fn login(state: &AppState, login_identifier: &str, password: &str) -> Option<String> {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let request = Request::post("/api/v1/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({ "login_identifier": login_identifier, "password": password })
                .to_string(),
        ))
        .unwrap();

    let response = aicte_apaar_api::routes::router(state.clone())
        .oneshot(request)
        .await
        .ok()?;
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .ok()?;
    if !status.is_success() {
        return None;
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    value["access_token"].as_str().map(|s| s.to_string())
}

/// The router under test.
pub fn app(state: AppState) -> axum::Router {
    aicte_apaar_api::routes::router(state)
}

/// Convenience: a GET with a bearer token.
pub fn get_request(path: &str, token: &str) -> axum::http::Request<axum::body::Body> {
    axum::http::Request::get(path)
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap()
}
