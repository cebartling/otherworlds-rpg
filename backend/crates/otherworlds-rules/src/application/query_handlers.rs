//! Query handlers for the Rules & Resolution context.
//!
//! This module contains query handlers that reconstitute aggregates
//! from stored events and return read-only view DTOs.

use otherworlds_core::error::DomainError;
use otherworlds_core::repository::EventRepository;
use serde::Serialize;
use uuid::Uuid;

use crate::application::command_handlers;

/// Read-only view of a resolution aggregate.
#[derive(Debug, Serialize)]
pub struct ResolutionView {
    /// The resolution identifier.
    pub resolution_id: Uuid,
    /// Intent IDs resolved during this resolution.
    pub intent_ids: Vec<Uuid>,
    /// Check IDs performed during this resolution.
    pub check_ids: Vec<Uuid>,
    /// Current version (event count).
    pub version: i64,
}

/// Retrieves a resolution by its aggregate ID.
///
/// # Errors
///
/// Returns `DomainError::AggregateNotFound` if no events exist for the ID.
/// Returns `DomainError::Infrastructure` if event deserialization fails.
pub async fn get_resolution_by_id(
    resolution_id: Uuid,
    repo: &dyn EventRepository,
) -> Result<ResolutionView, DomainError> {
    let stored_events = repo.load_events(resolution_id).await?;
    if stored_events.is_empty() {
        return Err(DomainError::AggregateNotFound(resolution_id));
    }
    let resolution = command_handlers::reconstitute(resolution_id, &stored_events)?;
    Ok(ResolutionView {
        resolution_id,
        intent_ids: resolution.intent_ids.clone(),
        check_ids: resolution.check_ids.clone(),
        version: resolution.version,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::StoredEvent;
    use uuid::Uuid;

    use crate::application::query_handlers::get_resolution_by_id;
    use crate::domain::events::{IntentResolved, RulesEventKind};
    use otherworlds_test_support::{EmptyEventRepository, RecordingEventRepository};

    #[tokio::test]
    async fn test_get_resolution_by_id_returns_view_with_state() {
        // Arrange
        let resolution_id = Uuid::new_v4();
        let intent_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        let events = vec![StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: resolution_id,
            event_type: "rules.intent_resolved".to_owned(),
            payload: serde_json::to_value(RulesEventKind::IntentResolved(IntentResolved {
                resolution_id,
                intent_id,
            }))
            .unwrap(),
            sequence_number: 1,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }];
        let repo = RecordingEventRepository::new(Ok(events));

        // Act
        let view = get_resolution_by_id(resolution_id, &repo).await.unwrap();

        // Assert
        assert_eq!(view.resolution_id, resolution_id);
        assert_eq!(view.intent_ids, vec![intent_id]);
        assert!(view.check_ids.is_empty());
        assert_eq!(view.version, 1);
    }

    #[tokio::test]
    async fn test_get_resolution_by_id_returns_not_found_when_no_events() {
        // Arrange
        let resolution_id = Uuid::new_v4();
        let repo = EmptyEventRepository;

        // Act
        let result = get_resolution_by_id(resolution_id, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, resolution_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }
}
