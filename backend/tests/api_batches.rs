//! Batch ingestion tests.
//!
//! Uploading the seeded demo batch must produce exactly the expected findings
//! and no row may be dropped silently.

mod common;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

fn multipart_body(rows: &str) -> Body {
    let boundary = "----apaartestboundary";
    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"demo.csv\"\r\n\
         Content-Type: text/csv\r\n\r\n\
         {rows}\r\n\
         --{boundary}--\r\n"
    );
    Body::from(body)
}

fn upload_request(body: Body) -> Request<Body> {
    Request::post("/api/v1/batches")
        .header(
            "content-type",
            "multipart/form-data; boundary=----apaartestboundary",
        )
        .body(body)
        .unwrap()
}

#[tokio::test]
async fn upload_reports_row_level_findings_and_keeps_every_row() {
    let Some(state) = common::test_state().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };

    let Some(token) = common::login(&state, "alpha.operator", "DemoPass123!").await else {
        eprintln!("skipped: demo users are not seeded");
        return;
    };

    let rows = "apaar_id,institution_student_id,full_name,programme_code,academic_year,admission_date,seat_category,admission_status\n\
                APAAR-TEST-0001,ALPHA-T1,Test One,CSE-01,2026-27,2026-07-01,REGULAR,ACTIVE\n\
                ,ALPHA-T2,Test Two,CSE-01,2026-27,2026-07-02,REGULAR,ACTIVE\n\
                APAAR-TEST-0001,ALPHA-T3,Test Three,CSE-01,2026-27,2026-07-03,REGULAR,ACTIVE\n\
                APAAR-TEST-0004,ALPHA-T4,Test Four,CSE-01,2026-27,05-07-2026,REGULAR,ACTIVE\n";

    let mut request = upload_request(multipart_body(rows));
    request
        .headers_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());

    let response = common::app(state.clone()).oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let batch: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Every row is counted - nothing is silently dropped.
    assert_eq!(batch["counts"]["rows"], 4);
    // One clean row, one missing APAAR id, one duplicate, one bad date.
    assert_eq!(batch["counts"]["accepted"], 1);
    assert_eq!(batch["counts"]["rejected"], 2);
    assert_eq!(batch["counts"]["needs_review"], 1);

    let batch_id = batch["id"].as_str().unwrap();

    // The findings must carry stable reason codes.
    let mut request = common::get_request(&format!("/api/v1/batches/{batch_id}/issues"), &token);
    request
        .headers_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    let response = common::app(state).oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), 256 * 1024).await.unwrap();
    let issues: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let codes: Vec<&str> = issues
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["rule_code"].as_str().unwrap())
        .collect();

    assert!(codes.contains(&"APAAR_ID_MISSING"));
    assert!(codes.contains(&"DUPLICATE_APAAR_IN_BATCH"));
    assert!(codes.contains(&"ADMISSION_DATE_INVALID"));
}

#[tokio::test]
async fn a_file_missing_a_required_column_is_rejected_with_an_actionable_error() {
    let Some(state) = common::test_state().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };

    let Some(token) = common::login(&state, "alpha.operator", "DemoPass123!").await else {
        eprintln!("skipped: demo users are not seeded");
        return;
    };

    let rows = "apaar_id,full_name\nAPAAR-TEST-9001,Test\n";
    let mut request = upload_request(multipart_body(rows));
    request
        .headers_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());

    let response = common::app(state).oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let message = value["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("missing the required column"),
        "the error must say which column is missing: {message}"
    );
}

#[tokio::test]
async fn a_data_operator_cannot_commit_a_batch() {
    let Some(state) = common::test_state().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };

    let Some(token) = common::login(&state, "alpha.operator", "DemoPass123!").await else {
        eprintln!("skipped: demo users are not seeded");
        return;
    };

    let request = Request::post("/api/v1/batches/00000000-0000-4000-8000-000000000000/commit")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let response = common::app(state).oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
