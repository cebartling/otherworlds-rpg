//! Domain error types.

use thiserror::Error;
use uuid::Uuid;

/// Top-level domain error type.
#[derive(Debug, Error)]
pub enum DomainError {
    /// An aggregate was not found.
    #[error("aggregate not found: {0}")]
    AggregateNotFound(Uuid),

    /// Optimistic concurrency conflict.
    #[error("concurrency conflict on aggregate {aggregate_id}: expected version {expected}, found {actual}")]
    ConcurrencyConflict {
        /// The aggregate that had the conflict.
        aggregate_id: Uuid,
        /// The expected version.
        expected: i64,
        /// The actual version found.
        actual: i64,
    },

    /// A validation error in domain logic.
    #[error("validation error: {0}")]
    Validation(String),

    /// An infrastructure/persistence error.
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
