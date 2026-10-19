pub mod policy;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;
use policy::{AuthenticatedUser, Role};

/// JWT payload. Deliberately minimal: no personal data, no scope list that
/// could be tampered with by the client.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    /// User id.
    pub sub: String,
    pub role: String,
    /// Institution scope. `None` for global roles.
    pub institution_id: Option<Uuid>,
    pub name: String,
    pub iat: i64,
    pub exp: i64,
}

pub fn encode_token(
    secret: &str,
    user: &AuthenticatedUser,
    ttl_minutes: i64,
) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user.id.to_string(),
        role: user.role.as_str().to_string(),
        institution_id: user.institution_id,
        name: user.display_name.clone(),
        iat: now,
        exp: now + ttl_minutes * 60,
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| AppError::internal(anyhow::anyhow!("token encoding failed")))
}

pub fn decode_token(secret: &str, token: &str) -> Result<Claims, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|err| match err.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
            AppError::unauthorized("Your session has expired. Sign in again.")
        }
        _ => AppError::unauthorized("Invalid authentication token."),
    })
}

/// Hashes a password with argon2id. Plaintext passwords are never stored.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| AppError::internal(anyhow::anyhow!("password hashing failed")))
}

/// Verifies a password against a stored argon2 hash.
pub fn verify_password(password: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

/// Extractor that requires a valid bearer token.
///
/// The role and institution scope come from the signed token only.
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                AppError::unauthorized("Authentication required. Provide a bearer token.")
            })?;

        let token = header
            .strip_prefix("Bearer ")
            .or_else(|| header.strip_prefix("bearer "))
            .ok_or_else(|| {
                AppError::unauthorized("Authentication required. Provide a bearer token.")
            })?
            .trim();

        let claims = decode_token(&state.config.jwt_secret, token)?;
        let role = Role::parse(&claims.role)
            .ok_or_else(|| AppError::unauthorized("The token carries an unknown role."))?;

        Ok(AuthenticatedUser {
            id: Uuid::parse_str(&claims.sub)
                .map_err(|_| AppError::unauthorized("The token carries an invalid subject."))?,
            display_name: claims.name,
            role,
            institution_id: claims.institution_id,
        })
    }
}
