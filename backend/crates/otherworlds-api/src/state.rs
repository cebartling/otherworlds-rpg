//! Shared application state.

use sqlx::PgPool;

/// Application state shared across all request handlers.
#[derive(Debug, Clone)]
pub struct AppState {
    /// PostgreSQL connection pool.
    pub db_pool: PgPool,
}

impl AppState {
    /// Create new application state.
    #[must_use]
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}
