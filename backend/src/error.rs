use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use serde_json::json;

/// Standardised API error.
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    pub fn bad_request(code: &'static str, msg: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, msg)
    }
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "unauthorized", msg)
    }
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, "forbidden", msg)
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", msg)
    }
    pub fn conflict(code: &'static str, msg: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, code, msg)
    }
    pub fn rate_limited(code: &'static str, msg: impl Into<String>) -> Self {
        Self::new(StatusCode::TOO_MANY_REQUESTS, code, msg)
    }
    pub fn payload_too_large(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::PAYLOAD_TOO_LARGE, "body_too_large", msg)
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", msg)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = json!({
            "status": "error",
            "code": self.code,
            "error": self.message,
        });
        (self.status, Json(body)).into_response()
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        tracing::error!("internal error: {e:#}");
        ApiError::internal("internal server error")
    }
}

impl From<mongodb::error::Error> for ApiError {
    fn from(e: mongodb::error::Error) -> Self {
        tracing::error!("mongo error: {e}");
        ApiError::internal("database error")
    }
}

impl From<redis::RedisError> for ApiError {
    fn from(e: redis::RedisError) -> Self {
        tracing::error!("redis error: {e}");
        ApiError::internal("cache error")
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
