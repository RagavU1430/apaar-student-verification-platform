use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::auth::policy::{AuthenticatedUser, Role};
use crate::auth::{decode_token, encode_token, verify_password};
use crate::error::{AppError, AppResult};
use crate::repositories::users;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub login_identifier: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
    pub user: UserProfile,
}

#[derive(Serialize)]
pub struct UserProfile {
    pub id: String,
    pub display_name: String,
    pub role: String,
    pub institution_id: Option<String>,
    pub institution_name: Option<String>,
    pub permissions: Vec<String>,
}

/// Signs a user in.
///
/// Failure is deliberately indistinguishable between "unknown user" and
/// "wrong password" so the endpoint cannot be used to enumerate accounts.
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    let row = users::find_by_login(&state.pool, request.login_identifier.trim()).await?;

    let Some(row) = row else {
        return Err(AppError::unauthorized("Invalid credentials."));
    };

    if !verify_password(&request.password, &row.password_hash) {
        return Err(AppError::unauthorized("Invalid credentials."));
    }

    if row.status != "ACTIVE" {
        return Err(AppError::unauthorized("This account is not active."));
    }

    let Some(role) = Role::parse(&row.role) else {
        return Err(AppError::internal(anyhow::anyhow!("unknown role in database")));
    };

    let user = AuthenticatedUser {
        id: row.id,
        display_name: row.display_name.clone(),
        role,
        institution_id: row.institution_id,
    };

    let institution_name = match row.institution_id {
        Some(id) => crate::repositories::institutions::get(&state.pool, id)
            .await?
            .map(|i| i.name),
        None => None,
    };

    let token = encode_token(
        &state.config.jwt_secret,
        &user,
        state.config.jwt_ttl_minutes,
    )?;

    Ok(Json(LoginResponse {
        access_token: token,
        token_type: "Bearer",
        expires_in: state.config.jwt_ttl_minutes * 60,
        user: UserProfile {
            id: user.id.to_string(),
            display_name: user.display_name.clone(),
            role: user.role.as_str().to_string(),
            institution_id: user.institution_id.map(|i| i.to_string()),
            institution_name,
            permissions: permissions_for(user.role),
        },
    }))
}

/// Returns the identity, role and scope of the current caller.
pub async fn me(user: AuthenticatedUser) -> AppResult<Json<UserProfile>> {
    Ok(Json(UserProfile {
        id: user.id.to_string(),
        display_name: user.display_name.clone(),
        role: user.role.as_str().to_string(),
        institution_id: user.institution_id.map(|i| i.to_string()),
        institution_name: None,
        permissions: permissions_for(user.role),
    }))
}

/// The permission list mirrors the server-side policy. The frontend uses it
/// only to decide what to show - the backend always re-checks.
fn permissions_for(role: Role) -> Vec<String> {
    let mut out = Vec::new();
    if role.can_upload() {
        out.push("batch:upload".to_string());
    }
    if role.can_commit() {
        out.push("batch:commit".to_string());
    }
    if role.can_view_all_institutions() {
        out.push("institutions:read_all".to_string());
        out.push("dashboard:aicte".to_string());
        out.push("issues:review".to_string());
    }
    if role.can_review() {
        out.push("issues:review".to_string());
    }
    if role.can_verify() {
        out.push("verification:request".to_string());
    }
    out
}

/// Decodes a token for tests and diagnostics. Not exposed over HTTP.
pub fn decode_for_diagnostics(secret: &str, token: &str) -> AppResult<serde_json::Value> {
    let claims = decode_token(secret, token)?;
    Ok(serde_json::json!({
        "sub": claims.sub,
        "role": claims.role,
        "institution_id": claims.institution_id,
    }))
}
