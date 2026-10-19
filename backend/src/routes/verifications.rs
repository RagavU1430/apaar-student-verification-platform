use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::policy::AuthenticatedUser;
use crate::domain::verification::{
    DisclosedField, VerificationOutcome, VerificationResult, default_limitations, policy_for,
};
use crate::error::{AppError, AppResult};
use crate::integrations::SimulatedAcademicIdentityVerifier;
use crate::integrations::AcademicIdentityVerifier;
use crate::repositories::{admissions, institutions, students};
use crate::services::audit;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct VerificationRequest {
    /// One of the configured purposes.
    pub purpose: String,
    /// The APAAR id being checked, or an institution student id.
    pub record_ref: String,
    /// Optional institution scope for a verifier that is assigned to one.
    pub institution_id: Option<String>,
}

#[derive(Serialize)]
pub struct VerificationResponse {
    pub outcome: String,
    pub fields_disclosed: Vec<DisclosedField>,
    pub source: String,
    pub verified_at: chrono::DateTime<chrono::Utc>,
    pub limitations: Vec<String>,
    /// Always true in the prototype: no live APAAR source is consulted.
    pub simulated: bool,
}

/// Purpose-limited record verification.
///
/// The verifier receives only the fields allowed for the declared purpose. The
/// request and the disclosed fields are recorded in the audit trail.
pub async fn verify(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<VerificationRequest>,
) -> AppResult<Json<VerificationResponse>> {
    if !user.role.can_verify() {
        audit::record_denied(
            &state.pool,
            &user,
            "verification.request",
            "verification_request",
            None,
            user.institution_id,
            "role is not allowed to request verifications",
        )
        .await?;
        return Err(AppError::forbidden(
            "Your role cannot request verifications.",
        ));
    }

    let Some(policy) = policy_for(request.purpose.trim()) else {
        return Err(AppError::bad_request(
            "Unknown verification purpose. Allowed purposes are listed in the API contract.",
        ));
    };

    let record_ref = request.record_ref.trim();
    if record_ref.is_empty() {
        return Err(AppError::bad_request("A record reference is required."));
    }

    // The external adapter is simulated in the prototype. It is never
    // presented as an authoritative APAAR check.
    let external = SimulatedAcademicIdentityVerifier.verify(record_ref);

    let institution_id = match request.institution_id.as_deref() {
        Some(raw) => Uuid::parse_str(raw).ok(),
        None => user.institution_id,
    };

    let mut fields: Vec<DisclosedField> = Vec::new();
    let outcome = match institution_id {
        Some(institution_id) => {
            match students::find_institution_student(&state.pool, record_ref, institution_id).await? {
                Some(identity_id) => {
                    let (rows, _) = admissions::search(
                        &state.pool,
                        crate::auth::policy::ScopeFilter::Institution(institution_id),
                        &admissions::AdmissionFilters::default(),
                        10,
                        0,
                    )
                    .await?;
                    let _ = identity_id;

                    let match_row = rows
                        .iter()
                        .find(|r| r.institution_student_id == record_ref || r.apaar_id_last4 == record_ref);

                    match match_row {
                        Some(row) => {
                            if row.status == "ACTIVE" || row.status == "ENROLLED" {
                                for name in policy.allowed_fields {
                                    if let Some(value) = disclosed_value(name, row) {
                                        fields.push(DisclosedField {
                                            name: name.to_string(),
                                            value,
                                        });
                                    }
                                }
                                VerificationOutcome::VerifiedAgainstPlatformRecords
                            } else {
                                fields.push(DisclosedField {
                                    name: "record_status".to_string(),
                                    value: row.status.clone(),
                                });
                                VerificationOutcome::NeedsReview
                            }
                        }
                        None => VerificationOutcome::NotFound,
                    }
                }
                None => VerificationOutcome::NotFound,
            }
        }
        None => VerificationOutcome::NotFound,
    };

    let institutions_seen = institutions::list_visible(&state.pool, user.scope_filter()).await?;

    sqlx::query(
        "INSERT INTO verification_requests
            (id, requester_id, purpose, record_ref, fields_disclosed, outcome, source, simulated)
         VALUES ($1,$2,$3,$4,$5,$6,$7,TRUE)",
    )
    .bind(Uuid::new_v4())
    .bind(user.id)
    .bind(policy.purpose)
    .bind(record_ref)
    .bind(serde_json::to_value(&fields).unwrap_or(serde_json::Value::Array(vec![])))
    .bind(outcome.as_str())
    .bind(&external.source)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::internal(e))?;

    audit::record(
        &state.pool,
        Some(&user),
        "verification.request",
        "verification_request",
        None,
        user.institution_id,
        outcome.as_str(),
        None,
        Some(format!(
            "purpose={} disclosed={}",
            policy.purpose,
            fields
                .iter()
                .map(|f| f.name.as_str())
                .collect::<Vec<_>>()
                .join(",")
        )),
    )
    .await?;

    let _ = institutions_seen;

    Ok(Json(VerificationResponse {
        outcome: outcome.as_str().to_string(),
        fields_disclosed: fields,
        source: external.source,
        verified_at: chrono::Utc::now(),
        limitations: default_limitations(external.simulated),
        simulated: external.simulated,
    }))
}

fn disclosed_value(name: &str, row: &admissions::AdmissionView) -> Option<String> {
    match name {
        "record_status" => Some(row.status.clone()),
        "academic_year" => Some(row.academic_year.clone()),
        "programme_code" => Some(row.programme_code.clone()),
        "admission_status" => Some(row.status.clone()),
        _ => None,
    }
}
