use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use serde_json::{json, Value};

/// Stable, machine-readable error codes (BUILD_PLAN.md section 5).
///
/// Codes are part of the API contract: they may be renamed only with a
/// versioned change to the API.
#[derive(Debug, thiserror::Error)]
pub enum ErrorCode {
    #[error("INVALID_REQUEST")]
    InvalidRequest,
    #[error("AUTHENTICATION_FAILED")]
    AuthenticationFailed,
    #[error("TOKEN_EXPIRED")]
    TokenExpired,
    #[error("PERMISSION_DENIED")]
    PermissionDenied,
    #[error("NOT_FOUND")]
    NotFound,
    #[error("CONFLICT")]
    Conflict,
    #[error("PAYLOAD_TOO_LARGE")]
    PayloadTooLarge,
    #[error("UNPROCESSABLE_ROW")]
    UnprocessableRow,
    #[error("RATE_LIMITED")]
    RateLimited,
    #[error("INTERNAL_ERROR")]
    Internal,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::InvalidRequest => "INVALID_REQUEST",
            ErrorCode::AuthenticationFailed => "AUTHENTICATION_FAILED",
            ErrorCode::TokenExpired => "TOKEN_EXPIRED",
            ErrorCode::PermissionDenied => "PERMISSION_DENIED",
            ErrorCode::NotFound => "NOT_FOUND",
            ErrorCode::Conflict => "CONFLICT",
            ErrorCode::PayloadTooLarge => "PAYLOAD_TOO_LARGE",
            ErrorCode::UnprocessableRow => "UNPROCESSABLE_ROW",
            ErrorCode::RateLimited => "RATE_LIMITED",
            ErrorCode::Internal => "INTERNAL_ERROR",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            ErrorCode::InvalidRequest => StatusCode::BAD_REQUEST,
            ErrorCode::AuthenticationFailed => StatusCode::UNAUTHORIZED,
            ErrorCode::TokenExpired => StatusCode::UNAUTHORIZED,
            ErrorCode::PermissionDenied => StatusCode::FORBIDDEN,
            ErrorCode::NotFound => StatusCode::NOT_FOUND,
            ErrorCode::Conflict => StatusCode::CONFLICT,
            ErrorCode::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            ErrorCode::UnprocessableRow => StatusCode::UNPROCESSABLE_ENTITY,
            ErrorCode::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            ErrorCode::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Application error. The inner cause is never rendered to a client.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    PayloadTooLarge(String),
    #[error("{code}: {message}")]
    Row { code: String, message: String },
    #[error("{0}")]
    Internal(anyhow::Error),
}

impl AppError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        AppError::BadRequest(msg.into())
    }
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        AppError::Unauthorized(msg.into())
    }
    pub fn forbidden(msg: impl Into<String>) -> Self {
        AppError::Forbidden(msg.into())
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        AppError::NotFound(msg.into())
    }
    pub fn conflict(msg: impl Into<String>) -> Self {
        AppError::Conflict(msg.into())
    }
    pub fn row(code: impl Into<String>, message: impl Into<String>) -> Self {
        AppError::Row {
            code: code.into(),
            message: message.into(),
        }
    }
    /// Wrap an internal failure. The source is logged, never returned.
    pub fn internal(err: impl Into<anyhow::Error>) -> Self {
        AppError::Internal(err.into())
    }

    fn parts(&self) -> (ErrorCode, String) {
        match self {
            AppError::BadRequest(m) => (ErrorCode::InvalidRequest, m.clone()),
            AppError::Unauthorized(m) => (ErrorCode::AuthenticationFailed, m.clone()),
            AppError::Forbidden(m) => (ErrorCode::PermissionDenied, m.clone()),
            AppError::NotFound(m) => (ErrorCode::NotFound, m.clone()),
            AppError::Conflict(m) => (ErrorCode::Conflict, m.clone()),
            AppError::PayloadTooLarge(m) => (ErrorCode::PayloadTooLarge, m.clone()),
            AppError::Row { code, message } => {
                (ErrorCode::UnprocessableRow, format!("{code}: {message}"))
            }
            AppError::Internal(_) => (
                ErrorCode::Internal,
                "An unexpected error occurred.".to_string(),
            ),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (code, message) = self.parts();
        if let AppError::Internal(err) = &self {
            tracing::error!(error = ?err, "internal error");
        }
        let body = ErrorEnvelope {
            error: ErrorBody {
                code: code.as_str().to_string(),
                message,
                details: Value::Null,
            },
        };
        (code.status(), Json(body)).into_response()
    }
}

#[derive(Serialize)]
pub struct ErrorEnvelope {
    pub error: ErrorBody,
}

#[derive(Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub details: Value,
}

pub type AppResult<T> = Result<T, AppError>;

/// Convenience for building a JSON error body in tests / handlers.
pub fn error_body(code: &str, message: &str) -> Value {
    json!({ "error": { "code": code, "message": message, "details": null } })
}
