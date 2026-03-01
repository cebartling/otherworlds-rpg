//! Shared application state.

use std::sync::{Arc, Mutex};

use sqlx::PgPool;

use otherworlds_core::clock::Clock;
use otherworlds_core::repository::EventRepository;
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
    /// Event repository for persisting and loading domain events.
    pub event_repository: Arc<dyn EventRepository>,
}

impl AppState {
    /// Create new application state.
    #[must_use]
    pub fn new(
        db_pool: PgPool,
        clock: Arc<dyn Clock + Send + Sync>,
        rng: Arc<Mutex<dyn DeterministicRng + Send>>,
        event_repository: Arc<dyn EventRepository>,
    ) -> Self {
        Self {
            db_pool,
            clock,
            rng,
            event_repository,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use otherworlds_test_support::{EmptyEventRepository, FixedClock, MockRng};

    #[tokio::test]
    async fn test_app_state_holds_event_repository() {
        let pool = PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let clock: Arc<dyn Clock + Send + Sync> =
            Arc::new(FixedClock(chrono::DateTime::UNIX_EPOCH));
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let event_repository: Arc<dyn EventRepository> = Arc::new(EmptyEventRepository);

        let state = AppState::new(pool, clock, rng, event_repository.clone());

        // Verify event_repository is accessible and is the same instance.
        assert!(Arc::ptr_eq(&state.event_repository, &event_repository));
    }
}
