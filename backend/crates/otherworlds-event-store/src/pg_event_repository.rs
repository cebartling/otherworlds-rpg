//! Event Store — `PostgreSQL` `EventRepository` implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::{debug, instrument, warn};
use uuid::Uuid;

use otherworlds_core::error::DomainError;
use otherworlds_core::repository::{EventRepository, StoredEvent};

/// `PostgreSQL` unique-violation error code.
const UNIQUE_VIOLATION: &str = "23505";

/// Internal row type for mapping `sqlx::FromRow` results.
#[derive(sqlx::FromRow)]
struct StoredEventRow {
    event_id: Uuid,
    aggregate_id: Uuid,
    event_type: String,
    payload: serde_json::Value,
    sequence_number: i64,
    correlation_id: Uuid,
    causation_id: Uuid,
    occurred_at: DateTime<Utc>,
}

impl From<StoredEventRow> for StoredEvent {
    fn from(row: StoredEventRow) -> Self {
        Self {
            event_id: row.event_id,
            aggregate_id: row.aggregate_id,
            event_type: row.event_type,
            payload: row.payload,
            sequence_number: row.sequence_number,
            correlation_id: row.correlation_id,
            causation_id: row.causation_id,
            occurred_at: row.occurred_at,
        }
    }
}

/// Returns `true` if the sqlx error is a `PostgreSQL` unique-violation.
fn is_unique_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        return db_err.code().as_deref() == Some(UNIQUE_VIOLATION);
    }
    false
}

/// Queries the current max sequence number for an aggregate.
async fn current_version(pool: &PgPool, aggregate_id: Uuid) -> Result<i64, DomainError> {
    let row: (Option<i64>,) =
        sqlx::query_as("SELECT MAX(sequence_number) FROM domain_events WHERE aggregate_id = $1")
            .bind(aggregate_id)
            .fetch_one(pool)
            .await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

    Ok(row.0.unwrap_or(0))
}

/// Maps a sqlx error to the appropriate `DomainError`.
///
/// On unique-violation, queries the current version from the pool (not the
/// aborted transaction) to populate the `ConcurrencyConflict` variant.
async fn map_sqlx_error(
    err: sqlx::Error,
    pool: &PgPool,
    aggregate_id: Uuid,
    expected_version: i64,
) -> DomainError {
    if is_unique_violation(&err) {
        match current_version(pool, aggregate_id).await {
            Ok(actual) => {
                return DomainError::ConcurrencyConflict {
                    aggregate_id,
                    expected: expected_version,
                    actual,
                };
            }
            Err(version_err) => {
                warn!(
                    %aggregate_id,
                    "failed to query current version after unique violation: {version_err}"
                );
                return DomainError::Infrastructure(format!(
                    "unique violation on aggregate {aggregate_id}, \
                     but failed to determine current version: {version_err}"
                ));
            }
        }
    }
    DomainError::Infrastructure(err.to_string())
}

/// `PostgreSQL`-backed event repository.
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
    #[instrument(skip(self), fields(%aggregate_id))]
    async fn load_events(&self, aggregate_id: Uuid) -> Result<Vec<StoredEvent>, DomainError> {
        let rows: Vec<StoredEventRow> = sqlx::query_as(
            "SELECT event_id, aggregate_id, event_type, payload, \
                    sequence_number, correlation_id, causation_id, occurred_at \
             FROM domain_events \
             WHERE aggregate_id = $1 \
             ORDER BY sequence_number ASC",
        )
        .bind(aggregate_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        debug!(event_count = rows.len(), "loaded events for aggregate");

        Ok(rows.into_iter().map(StoredEvent::from).collect())
    }

    #[instrument(skip(self, events), fields(%aggregate_id, %expected_version, event_count = events.len()))]
    async fn append_events(
        &self,
        aggregate_id: Uuid,
        expected_version: i64,
        events: &[StoredEvent],
    ) -> Result<(), DomainError> {
        if events.is_empty() {
            return Ok(());
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        // Proactive optimistic concurrency check: verify expected_version
        // matches the current max sequence_number within the transaction.
        let row: (Option<i64>,) = sqlx::query_as(
            "SELECT MAX(sequence_number) FROM domain_events WHERE aggregate_id = $1",
        )
        .bind(aggregate_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;
        let actual_version = row.0.unwrap_or(0);

        if actual_version != expected_version {
            return Err(DomainError::ConcurrencyConflict {
                aggregate_id,
                expected: expected_version,
                actual: actual_version,
            });
        }

        let event_ids: Vec<Uuid> = events.iter().map(|e| e.event_id).collect();
        let aggregate_ids: Vec<Uuid> = events.iter().map(|e| e.aggregate_id).collect();
        let event_types: Vec<&str> = events.iter().map(|e| e.event_type.as_str()).collect();
        let payloads: Vec<&serde_json::Value> = events.iter().map(|e| &e.payload).collect();
        let sequence_numbers: Vec<i64> = events.iter().map(|e| e.sequence_number).collect();
        let correlation_ids: Vec<Uuid> = events.iter().map(|e| e.correlation_id).collect();
        let causation_ids: Vec<Uuid> = events.iter().map(|e| e.causation_id).collect();
        let occurred_ats: Vec<DateTime<Utc>> = events.iter().map(|e| e.occurred_at).collect();

        let result = sqlx::query(
            "INSERT INTO domain_events \
                (event_id, aggregate_id, event_type, payload, \
                 sequence_number, correlation_id, causation_id, occurred_at) \
             SELECT * FROM UNNEST($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(&event_ids)
        .bind(&aggregate_ids)
        .bind(&event_types)
        .bind(&payloads)
        .bind(&sequence_numbers)
        .bind(&correlation_ids)
        .bind(&causation_ids)
        .bind(&occurred_ats)
        .execute(&mut *tx)
        .await;

        if let Err(err) = result {
            return Err(map_sqlx_error(err, &self.pool, aggregate_id, expected_version).await);
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        debug!("committed events for aggregate");

        Ok(())
    }
}
