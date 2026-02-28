//! Otherworlds RPG — API error types.

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
