//! Command handlers for the World State context.
//!
//! This module contains application-level command handler functions that
//! orchestrate domain logic: load aggregate, execute command, persist events.

use otherworlds_core::aggregate::AggregateRoot;
use otherworlds_core::clock::Clock;
use otherworlds_core::error::DomainError;
use otherworlds_core::event::DomainEvent;
use otherworlds_core::repository::{EventRepository, StoredEvent};
use uuid::Uuid;

use crate::domain::aggregates::WorldSnapshot;
use crate::domain::commands::{ApplyEffect, SetFlag, UpdateDisposition};
use crate::domain::events::WorldStateEvent;

fn to_stored_event(event: &WorldStateEvent) -> StoredEvent {
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

/// Reconstitutes a `WorldSnapshot` from stored events.
fn reconstitute(world_id: Uuid, existing_events: &[StoredEvent]) -> WorldSnapshot {
    let mut snapshot = WorldSnapshot::new(world_id);
    #[allow(clippy::cast_possible_wrap)]
    let version = existing_events.len() as i64;
    snapshot.version = version;
    snapshot
}

/// Handles the `ApplyEffect` command: reconstitutes the aggregate, applies
/// the effect, and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if validation fails or event persistence fails.
pub async fn handle_apply_effect(
    command: &ApplyEffect,
    clock: &dyn Clock,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    if command.fact_key.trim().is_empty() {
        return Err(DomainError::Validation("fact key must not be empty".into()));
    }

    let existing_events = repo.load_events(command.world_id).await?;
    let mut snapshot = reconstitute(command.world_id, &existing_events);

    snapshot.apply_effect(command.fact_key.clone(), command.correlation_id, clock);

    let stored_events: Vec<StoredEvent> = snapshot
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.world_id, snapshot.version, &stored_events)
        .await?;

    Ok(stored_events)
}

/// Handles the `SetFlag` command: reconstitutes the aggregate, sets the flag,
/// and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if validation fails or event persistence fails.
pub async fn handle_set_flag(
    command: &SetFlag,
    clock: &dyn Clock,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    if command.flag_key.trim().is_empty() {
        return Err(DomainError::Validation("flag key must not be empty".into()));
    }

    let existing_events = repo.load_events(command.world_id).await?;
    let mut snapshot = reconstitute(command.world_id, &existing_events);

    snapshot.set_flag(
        command.flag_key.clone(),
        command.value,
        command.correlation_id,
        clock,
    );

    let stored_events: Vec<StoredEvent> = snapshot
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.world_id, snapshot.version, &stored_events)
        .await?;

    Ok(stored_events)
}

/// Handles the `UpdateDisposition` command: reconstitutes the aggregate,
/// updates the disposition, and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if event loading or appending fails.
pub async fn handle_update_disposition(
    command: &UpdateDisposition,
    clock: &dyn Clock,
    repo: &dyn EventRepository,
) -> Result<Vec<StoredEvent>, DomainError> {
    let existing_events = repo.load_events(command.world_id).await?;
    let mut snapshot = reconstitute(command.world_id, &existing_events);

    snapshot.update_disposition(command.entity_id, command.correlation_id, clock);

    let stored_events: Vec<StoredEvent> = snapshot
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.world_id, snapshot.version, &stored_events)
        .await?;

    Ok(stored_events)
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Utc};
    use otherworlds_core::clock::Clock;
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::{EventRepository, StoredEvent};
    use std::sync::Mutex;
    use uuid::Uuid;

    use crate::application::command_handlers::{
        handle_apply_effect, handle_set_flag, handle_update_disposition,
    };
    use crate::domain::commands::{ApplyEffect, SetFlag, UpdateDisposition};
    use crate::domain::events::WorldStateEventKind;

    #[derive(Debug)]
    struct FixedClock(DateTime<Utc>);

    impl Clock for FixedClock {
        fn now(&self) -> DateTime<Utc> {
            self.0
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
    async fn test_handle_apply_effect_persists_world_fact_changed_event() {
        // Arrange
        let world_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let repo = MockEventRepository::new(Ok(Vec::new()));

        let command = ApplyEffect {
            correlation_id,
            world_id,
            fact_key: "quest_complete".to_owned(),
        };

        // Act
        let result = handle_apply_effect(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, world_id);
        assert_eq!(*expected_version, 0);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "world_state.world_fact_changed");
        assert_eq!(stored.aggregate_id, world_id);
        assert_eq!(stored.sequence_number, 1);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);

        let kind: WorldStateEventKind = serde_json::from_value(stored.payload.clone()).unwrap();
        match kind {
            WorldStateEventKind::WorldFactChanged(payload) => {
                assert_eq!(payload.world_id, world_id);
                assert_eq!(payload.fact_key, "quest_complete");
            }
            other => panic!("expected WorldFactChanged, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_set_flag_persists_flag_set_event() {
        // Arrange
        let world_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let repo = MockEventRepository::new(Ok(Vec::new()));

        let command = SetFlag {
            correlation_id,
            world_id,
            flag_key: "door_unlocked".to_owned(),
            value: true,
        };

        // Act
        let result = handle_set_flag(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, world_id);
        assert_eq!(*expected_version, 0);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "world_state.flag_set");
        assert_eq!(stored.aggregate_id, world_id);
        assert_eq!(stored.sequence_number, 1);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);

        let kind: WorldStateEventKind = serde_json::from_value(stored.payload.clone()).unwrap();
        match kind {
            WorldStateEventKind::FlagSet(payload) => {
                assert_eq!(payload.world_id, world_id);
                assert_eq!(payload.flag_key, "door_unlocked");
                assert!(payload.value);
            }
            other => panic!("expected FlagSet, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_update_disposition_persists_disposition_updated_event() {
        // Arrange
        let world_id = Uuid::new_v4();
        let entity_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let repo = MockEventRepository::new(Ok(Vec::new()));

        let command = UpdateDisposition {
            correlation_id,
            world_id,
            entity_id,
        };

        // Act
        let result = handle_update_disposition(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_ok());

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, world_id);
        assert_eq!(*expected_version, 0);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "world_state.disposition_updated");
        assert_eq!(stored.aggregate_id, world_id);
        assert_eq!(stored.sequence_number, 1);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);

        let kind: WorldStateEventKind = serde_json::from_value(stored.payload.clone()).unwrap();
        match kind {
            WorldStateEventKind::DispositionUpdated(payload) => {
                assert_eq!(payload.world_id, world_id);
                assert_eq!(payload.entity_id, entity_id);
            }
            other => panic!("expected DispositionUpdated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_apply_effect_rejects_empty_fact_key() {
        // Arrange
        let clock = FixedClock(Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap());
        let repo = MockEventRepository::new(Ok(Vec::new()));

        let command = ApplyEffect {
            correlation_id: Uuid::new_v4(),
            world_id: Uuid::new_v4(),
            fact_key: "  ".to_owned(),
        };

        // Act
        let result = handle_apply_effect(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "fact key must not be empty");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_set_flag_rejects_empty_flag_key() {
        // Arrange
        let clock = FixedClock(Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap());
        let repo = MockEventRepository::new(Ok(Vec::new()));

        let command = SetFlag {
            correlation_id: Uuid::new_v4(),
            world_id: Uuid::new_v4(),
            flag_key: String::new(),
            value: true,
        };

        // Act
        let result = handle_set_flag(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "flag key must not be empty");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
