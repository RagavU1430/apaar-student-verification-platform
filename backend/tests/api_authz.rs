//! Authorisation tests.
//!
//! These are the negative tests that matter most: a caller from one
//! institution must never reach another institution's data, and the attempt
//! must be recorded.

mod common;

use axum::body::to_bytes;
use axum::http::StatusCode;
use tower::ServiceExt;

#[tokio::test]
async fn unauthenticated_requests_are_rejected() {
    let Some(state) = common::test_state().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };

    let response = common::app(state)
        .oneshot(common::get_request(
            "/api/v1/dashboard/institution",
            "not-a-token",
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn institution_user_cannot_read_another_institutions_intakes() {
    let Some(state) = common::test_state().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };

    let Some(token) = common::login(&state, "alpha.operator", "DemoPass123!").await else {
        eprintln!("skipped: demo users are not seeded (run: cargo run --bin seed_users)");
        return;
    };

    // Beta institution id, from the seeded master data.
    let beta = "22222222-2222-4222-8222-222222222222";
    let response = common::app(state)
        .oneshot(common::get_request(
            &format!("/api/v1/institutions/{beta}/intakes"),
            &token,
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "a cross-institution read must be refused"
    );

    let body = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value["error"]["code"], "PERMISSION_DENIED");
    // The error must not leak the restricted configuration.
    assert!(body.windows(8).all(|w| w != b"regular_"));
}

#[tokio::test]
async fn institution_user_cannot_open_the_aicte_dashboard() {
    let Some(state) = common::test_state().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };

    let Some(token) = common::login(&state, "alpha.operator", "DemoPass123!").await else {
        eprintln!("skipped: demo users are not seeded");
        return;
    };

    let response = common::app(state)
        .oneshot(common::get_request("/api/v1/dashboard/aicte", &token))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn aicte_admin_can_open_the_aicte_dashboard() {
    let Some(state) = common::test_state().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };

    let Some(token) = common::login(&state, "aicte.admin", "DemoPass123!").await else {
        eprintln!("skipped: demo users are not seeded");
        return;
    };

    let response = common::app(state)
        .oneshot(common::get_request("/api/v1/dashboard/aicte", &token))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn invalid_credentials_do_not_reveal_whether_the_account_exists() {
    let Some(state) = common::test_state().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };

    let unknown = common::login(&state, "does-not-exist", "whatever").await;
    let wrong_password = common::login(&state, "alpha.operator", "wrong").await;

    assert!(unknown.is_none());
    assert!(wrong_password.is_none());
}
