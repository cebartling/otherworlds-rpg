//! `PostgreSQL` implementation of the `EventRepository` trait.

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use otherworlds_core::error::DomainError;
use otherworlds_core::repository::{EventRepository, StoredEvent};

/// PostgreSQL-backed event repository.
#[derive(Debug, Clone)]
pub struct PgEventRepository {
    pool: PgPool,
}

impl PgEventRepository {
    /// Creates a new `PgEventRepository`.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventRepository for PgEventRepository {
    async fn load_events(&self, _aggregate_id: Uuid) -> Result<Vec<StoredEvent>, DomainError> {
        todo!("PostgreSQL event loading will be implemented in the next phase")
    }

    async fn append_events(
        &self,
        _aggregate_id: Uuid,
        _expected_version: i64,
        _events: Vec<StoredEvent>,
    ) -> Result<(), DomainError> {
        todo!("PostgreSQL event appending will be implemented in the next phase")
    }
}
