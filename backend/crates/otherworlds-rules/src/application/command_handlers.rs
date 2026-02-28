//! Command handlers for the Rules & Resolution context.
//!
//! This module contains application-level command handler functions that
//! orchestrate domain logic: load aggregate, execute command, persist events.

use std::sync::Mutex;

use otherworlds_core::aggregate::AggregateRoot;
use otherworlds_core::clock::Clock;
use otherworlds_core::error::DomainError;
use otherworlds_core::event::DomainEvent;
use otherworlds_core::repository::{EventRepository, StoredEvent};
use otherworlds_core::rng::DeterministicRng;
use uuid::Uuid;

use crate::domain::aggregates::Resolution;
use crate::domain::commands::{PerformCheck, ResolveIntent};
use crate::domain::events::RulesEvent;

fn to_stored_event(event: &RulesEvent) -> StoredEvent {
    let meta = event.metadata();
    StoredEvent {
        event_id: meta.event_id,
        aggregate_id: meta.aggregate_id,
        event_type: event.event_type().to_owned(),
        payload: event.to_payload(),
        sequence_number: meta.sequence_number,
        correlation_id: meta.correlation_id,
        causation_id: meta.causation_id,
        occurred_at: meta.occurred_at,
    }
}

/// Reconstitutes a `Resolution` from stored events.
fn reconstitute(resolution_id: Uuid, existing_events: &[StoredEvent]) -> Resolution {
    let mut resolution = Resolution::new(resolution_id);
    #[allow(clippy::cast_possible_wrap)]
    let version = existing_events.len() as i64;
    resolution.version = version;
    resolution
}

/// Handles the `ResolveIntent` command: reconstitutes the aggregate, resolves
/// the intent, and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if event loading or appending fails.
pub async fn handle_resolve_intent(
    command: &ResolveIntent,
    clock: &dyn Clock,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    let existing_events = repo.load_events(command.intent_id).await?;
    let mut resolution = reconstitute(command.intent_id, &existing_events);

    resolution.resolve_intent(command.intent_id, command.correlation_id, clock);

    let stored_events: Vec<StoredEvent> = resolution
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.intent_id, resolution.version, &stored_events)
        .await?;

    Ok(stored_events)
}

/// Handles the `PerformCheck` command: reconstitutes the aggregate, performs
/// the check using the RNG, and persists the resulting events.
///
/// The `Mutex` is locked only around the synchronous domain method call to
/// avoid holding a `MutexGuard` across await points.
///
/// # Errors
///
/// Returns `DomainError` if event loading or appending fails.
pub async fn handle_perform_check(
    command: &PerformCheck,
    clock: &dyn Clock,
    rng: &Mutex<dyn DeterministicRng + Send>,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    let existing_events = repo.load_events(command.resolution_id).await?;
    let mut resolution = reconstitute(command.resolution_id, &existing_events);

    // Lock RNG only for the synchronous domain method — never across an await.
    {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        resolution.perform_check(command.correlation_id, clock, &mut *rng_guard);
    }

    let stored_events: Vec<StoredEvent> = resolution
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.resolution_id, resolution.version, &stored_events)
        .await?;

    Ok(stored_events)
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Utc};
    use otherworlds_core::clock::Clock;
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::{EventRepository, StoredEvent};
    use otherworlds_core::rng::DeterministicRng;
    use std::sync::Mutex;
    use uuid::Uuid;

    use crate::application::command_handlers::{handle_perform_check, handle_resolve_intent};
    use crate::domain::commands::{PerformCheck, ResolveIntent};

    #[derive(Debug)]
    struct FixedClock(DateTime<Utc>);

    impl Clock for FixedClock {
        fn now(&self) -> DateTime<Utc> {
            self.0
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
    struct MockEventRepository {
        load_result: Mutex<Option<Result<Vec<StoredEvent>, DomainError>>>,
        appended: Mutex<Vec<(Uuid, i64, Vec<StoredEvent>)>>,
    }

    impl MockEventRepository {
        fn new(load_result: Result<Vec<StoredEvent>, DomainError>) -> Self {
            Self {
                load_result: Mutex::new(Some(load_result)),
                appended: Mutex::new(Vec::new()),
            }
        }

        fn appended_events(&self) -> Vec<(Uuid, i64, Vec<StoredEvent>)> {
            self.appended.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl EventRepository for MockEventRepository {
        async fn load_events(&self, _aggregate_id: Uuid) -> Result<Vec<StoredEvent>, DomainError> {
            self.load_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or(Ok(Vec::new()))
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
    }

    #[tokio::test]
    async fn test_handle_resolve_intent_persists_intent_resolved_event() {
        // Arrange
        let intent_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let repo = MockEventRepository::new(Ok(Vec::new()));

        let command = ResolveIntent {
            correlation_id,
            intent_id,
        };

        // Act
        let result = handle_resolve_intent(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, intent_id);
        assert_eq!(*expected_version, 0);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "rules.intent_resolved");
        assert_eq!(stored.aggregate_id, intent_id);
        assert_eq!(stored.sequence_number, 1);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);
    }

    #[tokio::test]
    async fn test_handle_perform_check_persists_check_performed_event() {
        // Arrange
        let resolution_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Mutex<MockRng> = Mutex::new(MockRng);
        let rng: &Mutex<dyn DeterministicRng + Send> = &rng;
        let repo = MockEventRepository::new(Ok(Vec::new()));

        let command = PerformCheck {
            correlation_id,
            resolution_id,
        };

        // Act
        let result = handle_perform_check(&command, &clock, rng, &repo).await;

        // Assert
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, resolution_id);
        assert_eq!(*expected_version, 0);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "rules.check_performed");
        assert_eq!(stored.aggregate_id, resolution_id);
        assert_eq!(stored.sequence_number, 1);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);
    }
}
