//! Test repositories — mock `EventRepository` implementations for tests.

use std::sync::Mutex;

use async_trait::async_trait;
use otherworlds_core::error::DomainError;
use otherworlds_core::repository::{EventRepository, StoredEvent};
use uuid::Uuid;

/// An event repository that records all `load_events` and `append_events`
/// calls. Returns the configured result from `load_events` on every call and
/// always succeeds on `append_events`.
#[derive(Debug)]
pub struct RecordingEventRepository {
    load_result: Mutex<Vec<StoredEvent>>,
    appended: Mutex<Vec<(Uuid, i64, Vec<StoredEvent>)>>,
    aggregate_ids: Mutex<Vec<Uuid>>,
}

impl RecordingEventRepository {
    /// Create a new recording repository that will return `load_result` from
    /// every `load_events` call.
    ///
    /// # Errors
    ///
    /// If `load_result` is an `Err`, the error message is stored and returned
    /// on every `load_events` call. Use `FailingEventRepository` instead for
    /// the common "always fail" pattern.
    ///
    /// # Panics
    ///
    /// Panics if `load_result` is an `Err` — use `FailingEventRepository` for
    /// error scenarios.
    #[must_use]
    pub fn new(load_result: Result<Vec<StoredEvent>, DomainError>) -> Self {
        Self {
            load_result: Mutex::new(load_result.expect(
                "RecordingEventRepository::new does not accept Err; use FailingEventRepository",
            )),
            appended: Mutex::new(Vec::new()),
            aggregate_ids: Mutex::new(Vec::new()),
        }
    }

    /// Create a new recording repository with pre-configured aggregate IDs
    /// returned by `list_aggregate_ids`.
    ///
    /// # Panics
    ///
    /// Panics if `load_result` is `Err`. Use `FailingEventRepository` for
    /// error-path testing instead.
    #[must_use]
    pub fn with_aggregate_ids(
        load_result: Result<Vec<StoredEvent>, DomainError>,
        aggregate_ids: Vec<Uuid>,
    ) -> Self {
        Self {
            load_result: Mutex::new(load_result.expect(
                "RecordingEventRepository::with_aggregate_ids does not accept Err; use FailingEventRepository",
            )),
            appended: Mutex::new(Vec::new()),
            aggregate_ids: Mutex::new(aggregate_ids),
        }
    }

    /// Returns a snapshot of all events that were appended.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn appended_events(&self) -> Vec<(Uuid, i64, Vec<StoredEvent>)> {
        self.appended.lock().unwrap().clone()
    }
}

#[async_trait]
impl EventRepository for RecordingEventRepository {
    async fn load_events(&self, _aggregate_id: Uuid) -> Result<Vec<StoredEvent>, DomainError> {
        Ok(self.load_result.lock().unwrap().clone())
    }

    async fn append_events(
        &self,
        aggregate_id: Uuid,
        expected_version: i64,
        events: &[StoredEvent],
    ) -> Result<(), DomainError> {
        self.appended
            .lock()
            .unwrap()
            .push((aggregate_id, expected_version, events.to_vec()));
        Ok(())
    }

    async fn list_aggregate_ids(&self, _event_types: &[&str]) -> Result<Vec<Uuid>, DomainError> {
        Ok(self.aggregate_ids.lock().unwrap().clone())
    }
}

/// An event repository that always returns an empty event list and silently
/// accepts appends. Useful for testing "aggregate not found" scenarios and
/// creation commands.
#[derive(Debug)]
pub struct EmptyEventRepository;

#[async_trait]
impl EventRepository for EmptyEventRepository {
    async fn load_events(&self, _aggregate_id: Uuid) -> Result<Vec<StoredEvent>, DomainError> {
        Ok(vec![])
    }

    async fn append_events(
        &self,
        _aggregate_id: Uuid,
        _expected_version: i64,
        _events: &[StoredEvent],
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn list_aggregate_ids(&self, _event_types: &[&str]) -> Result<Vec<Uuid>, DomainError> {
        Ok(vec![])
    }
}

/// An event repository that always returns an infrastructure error. Useful for
/// testing error-handling paths.
#[derive(Debug)]
pub struct FailingEventRepository;

#[async_trait]
impl EventRepository for FailingEventRepository {
    async fn load_events(&self, _aggregate_id: Uuid) -> Result<Vec<StoredEvent>, DomainError> {
        Err(DomainError::Infrastructure("connection refused".into()))
    }

    async fn append_events(
        &self,
        _aggregate_id: Uuid,
        _expected_version: i64,
        _events: &[StoredEvent],
    ) -> Result<(), DomainError> {
        Err(DomainError::Infrastructure("connection refused".into()))
    }

    async fn list_aggregate_ids(&self, _event_types: &[&str]) -> Result<Vec<Uuid>, DomainError> {
        Err(DomainError::Infrastructure("connection refused".into()))
    }
}
