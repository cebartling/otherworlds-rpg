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

    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use otherworlds_core::repository::{EventRepository, StoredEvent};
    use uuid::Uuid;

    #[derive(Debug)]
    struct MockClock;

    impl Clock for MockClock {
        fn now(&self) -> DateTime<Utc> {
            DateTime::UNIX_EPOCH
        }
    }

    #[derive(Debug)]
    struct MockRng;

    impl DeterministicRng for MockRng {
        fn next_u32_range(&mut self, min: u32, _max: u32) -> u32 {
            min
        }

        fn next_f64(&mut self) -> f64 {
            0.0
        }
    }

    #[derive(Debug)]
    struct MockEventRepository;

    #[async_trait]
    impl EventRepository for MockEventRepository {
        async fn load_events(
            &self,
            _aggregate_id: Uuid,
        ) -> Result<Vec<StoredEvent>, otherworlds_core::error::DomainError> {
            Ok(vec![])
        }

        async fn append_events(
            &self,
            _aggregate_id: Uuid,
            _expected_version: i64,
            _events: &[StoredEvent],
        ) -> Result<(), otherworlds_core::error::DomainError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_app_state_holds_event_repository() {
        let pool = PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let clock: Arc<dyn Clock + Send + Sync> = Arc::new(MockClock);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let event_repository: Arc<dyn EventRepository> = Arc::new(MockEventRepository);

        let state = AppState::new(pool, clock, rng, event_repository.clone());

        // Verify event_repository is accessible and is the same instance.
        assert!(Arc::ptr_eq(&state.event_repository, &event_repository));
    }
}
