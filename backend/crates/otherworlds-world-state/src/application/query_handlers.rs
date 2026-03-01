//! Query handlers for the World State context.
//!
//! This module contains query handlers that reconstitute aggregates
//! from stored events and return read-only view DTOs.

use std::collections::HashMap;

use otherworlds_core::error::DomainError;
use otherworlds_core::repository::EventRepository;
use serde::Serialize;
use uuid::Uuid;

use crate::application::command_handlers;

/// Read-only view of a world snapshot aggregate.
#[derive(Debug, Serialize)]
pub struct WorldSnapshotView {
    /// The world snapshot identifier.
    pub world_id: Uuid,
    /// Fact keys that have been applied.
    pub facts: Vec<String>,
    /// Boolean flags set in the world.
    pub flags: HashMap<String, bool>,
    /// Entity IDs whose dispositions have been updated.
    pub disposition_entity_ids: Vec<Uuid>,
    /// Current version (event count).
    pub version: i64,
}

/// Retrieves a world snapshot by its aggregate ID.
///
/// # Errors
///
/// Returns `DomainError::AggregateNotFound` if no events exist for the ID.
/// Returns `DomainError::Infrastructure` if event deserialization fails.
pub async fn get_world_snapshot_by_id(
    world_id: Uuid,
    repo: &dyn EventRepository,
) -> Result<WorldSnapshotView, DomainError> {
    let stored_events = repo.load_events(world_id).await?;
    if stored_events.is_empty() {
        return Err(DomainError::AggregateNotFound(world_id));
    }
    let snapshot = command_handlers::reconstitute(world_id, &stored_events)?;
    Ok(WorldSnapshotView {
        world_id,
        facts: snapshot.facts.clone(),
        flags: snapshot.flags.clone(),
        disposition_entity_ids: snapshot.disposition_entity_ids.clone(),
        version: snapshot.version,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::StoredEvent;
    use uuid::Uuid;

    use crate::application::query_handlers::get_world_snapshot_by_id;
    use crate::domain::events::{
        FlagSet, WorldFactChanged, WorldStateEventKind,
    };
    use otherworlds_test_support::{EmptyEventRepository, RecordingEventRepository};

    #[tokio::test]
    async fn test_get_world_snapshot_by_id_returns_view_with_state() {
        // Arrange
        let world_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        let events = vec![
            StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: world_id,
                event_type: "world_state.world_fact_changed".to_owned(),
                payload: serde_json::to_value(WorldStateEventKind::WorldFactChanged(
                    WorldFactChanged {
                        world_id,
                        fact_key: "quest_complete".to_owned(),
                    },
                ))
                .unwrap(),
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: world_id,
                event_type: "world_state.flag_set".to_owned(),
                payload: serde_json::to_value(WorldStateEventKind::FlagSet(FlagSet {
                    world_id,
                    flag_key: "door_unlocked".to_owned(),
                    value: true,
                }))
                .unwrap(),
                sequence_number: 2,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
        ];
        let repo = RecordingEventRepository::new(Ok(events));

        // Act
        let view = get_world_snapshot_by_id(world_id, &repo).await.unwrap();

        // Assert
        assert_eq!(view.world_id, world_id);
        assert_eq!(view.facts, vec!["quest_complete".to_owned()]);
        assert_eq!(view.flags.get("door_unlocked"), Some(&true));
        assert!(view.disposition_entity_ids.is_empty());
        assert_eq!(view.version, 2);
    }

    #[tokio::test]
    async fn test_get_world_snapshot_by_id_returns_not_found_when_no_events() {
        // Arrange
        let world_id = Uuid::new_v4();
        let repo = EmptyEventRepository;

        // Act
        let result = get_world_snapshot_by_id(world_id, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, world_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }
}
