//! Otherworlds RPG — API error types.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use otherworlds_core::error::DomainError;
use serde::Serialize;
use thiserror::Error;

/// Startup and runtime errors for the API server.
#[derive(Debug, Error)]
pub enum AppError {
    /// A required environment variable is missing or invalid.
    #[error("configuration error: {0}")]
    Config(String),

    /// Database connection or pool error.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Network binding or I/O error.
    #[error("server error: {0}")]
    Server(#[from] std::io::Error),
}

/// JSON body returned for error responses.
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    /// Machine-readable error code.
    pub error: &'static str,
    /// Human-readable error message.
    pub message: String,
}

/// HTTP-layer wrapper around `DomainError` that implements `IntoResponse`.
#[derive(Debug)]
pub struct ApiError(pub DomainError);

impl From<DomainError> for ApiError {
    fn from(err: DomainError) -> Self {
        Self(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_code) = match &self.0 {
            DomainError::AggregateNotFound(_) => (StatusCode::NOT_FOUND, "aggregate_not_found"),
            DomainError::ConcurrencyConflict { .. } => {
                (StatusCode::CONFLICT, "concurrency_conflict")
            }
            DomainError::Validation(_) => (StatusCode::BAD_REQUEST, "validation_error"),
            DomainError::Infrastructure(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "infrastructure_error")
            }
        };

        let body = ErrorBody {
            error: error_code,
            message: self.0.to_string(),
        };

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use uuid::Uuid;

    fn status_of(err: DomainError) -> StatusCode {
        let response = ApiError(err).into_response();
        response.status()
    }

    #[test]
    fn test_aggregate_not_found_maps_to_404() {
        let id = Uuid::new_v4();
        assert_eq!(
            status_of(DomainError::AggregateNotFound(id)),
            StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn test_concurrency_conflict_maps_to_409() {
        assert_eq!(
            status_of(DomainError::ConcurrencyConflict {
                aggregate_id: Uuid::new_v4(),
                expected: 1,
                actual: 2,
            }),
            StatusCode::CONFLICT
        );
    }

    #[test]
    fn test_validation_maps_to_400() {
        assert_eq!(
            status_of(DomainError::Validation("bad input".into())),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn test_infrastructure_maps_to_500() {
        assert_eq!(
            status_of(DomainError::Infrastructure("db down".into())),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
