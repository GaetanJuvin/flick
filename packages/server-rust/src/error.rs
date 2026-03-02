use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{message}")]
    NotFound { entity: String, id: Option<String>, message: String },

    #[error("{0}")]
    Conflict(String),

    #[error("{0}")]
    Unauthorized(String),

    #[error("{0}")]
    Forbidden(String),

    #[error("{0}")]
    Validation(String),

    #[error("{message}")]
    RateLimited { message: String, retry_after: i64 },

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl AppError {
    pub fn not_found(entity: &str, id: Option<&str>) -> Self {
        let message = match id {
            Some(id) => format!("{} '{}' not found", entity, id),
            None => format!("{} not found", entity),
        };
        Self::NotFound {
            entity: entity.to_string(),
            id: id.map(|s| s.to_string()),
            message,
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized(message.into())
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden(message.into())
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn rate_limited(message: impl Into<String>, retry_after: i64) -> Self {
        Self::RateLimited {
            message: message.into(),
            retry_after,
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_code(&self) -> &str {
        match self {
            Self::NotFound { .. } => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::RateLimited { .. } => "RATE_LIMITED",
            Self::Internal(_) | Self::Database(_) => "INTERNAL_ERROR",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.error_code();

        // Log server errors
        match &self {
            Self::Internal(e) => tracing::error!(error = %e, "Internal server error"),
            Self::Database(e) => tracing::error!(error = %e, "Database error"),
            _ => {}
        }

        let message = match &self {
            Self::Internal(_) | Self::Database(_) => "Internal server error".to_string(),
            other => other.to_string(),
        };

        let body = json!({
            "error": {
                "code": code,
                "message": message,
            }
        });

        let mut response = (status, axum::Json(body)).into_response();

        // Add Retry-After header for rate limiting
        if let Self::RateLimited { retry_after, .. } = &self {
            response.headers_mut().insert(
                "Retry-After",
                retry_after.to_string().parse().unwrap(),
            );
        }

        response
    }
}
