//! Shared application state.

use std::sync::{Arc, Mutex};

use sqlx::PgPool;

use otherworlds_core::clock::Clock;
use otherworlds_core::rng::DeterministicRng;

/// Application state shared across all request handlers.
#[derive(Debug, Clone)]
pub struct AppState {
    /// `PostgreSQL` connection pool.
    pub db_pool: PgPool,
    /// Clock abstraction for deterministic time.
    pub clock: Arc<dyn Clock + Send + Sync>,
    /// RNG abstraction for deterministic randomness.
    pub rng: Arc<Mutex<dyn DeterministicRng + Send>>,
}

impl AppState {
    /// Create new application state.
    #[must_use]
    pub fn new(
        db_pool: PgPool,
        clock: Arc<dyn Clock + Send + Sync>,
        rng: Arc<Mutex<dyn DeterministicRng + Send>>,
    ) -> Self {
        Self {
            db_pool,
            clock,
            rng,
        }
    }
}
