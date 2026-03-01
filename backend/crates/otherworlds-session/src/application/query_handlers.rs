//! Query handlers for the Session & Progress context.
//!
//! This module contains query handlers that reconstitute aggregates
//! from stored events and return read-only view DTOs.

use otherworlds_core::error::DomainError;
use otherworlds_core::repository::EventRepository;
use serde::Serialize;
use uuid::Uuid;

use crate::application::command_handlers;

/// Read-only view of a campaign run aggregate.
///
/// Note: `branch_source` is intentionally omitted from this view.
/// Branching metadata is internal bookkeeping; consumers that need it
/// should use a dedicated branching query.
#[derive(Debug, Serialize)]
pub struct CampaignRunView {
    /// The campaign run identifier.
    pub run_id: Uuid,
    /// The campaign this run belongs to.
    pub campaign_id: Option<Uuid>,
    /// Checkpoint IDs created during this run.
    pub checkpoint_ids: Vec<Uuid>,
    /// Current version (event count).
    pub version: i64,
}

/// Retrieves a campaign run by its aggregate ID.
///
/// # Errors
///
/// Returns `DomainError::AggregateNotFound` if no events exist for the ID.
/// Returns `DomainError::Infrastructure` if event deserialization fails.
pub async fn get_campaign_run_by_id(
    run_id: Uuid,
    repo: &dyn EventRepository,
) -> Result<CampaignRunView, DomainError> {
    let stored_events = repo.load_events(run_id).await?;
    if stored_events.is_empty() {
        return Err(DomainError::AggregateNotFound(run_id));
    }
    let run = command_handlers::reconstitute(run_id, &stored_events)?;
    Ok(CampaignRunView {
        run_id,
        campaign_id: run.campaign_id,
        checkpoint_ids: run.checkpoint_ids.clone(),
        version: run.version,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::StoredEvent;
    use uuid::Uuid;

    use crate::application::query_handlers::get_campaign_run_by_id;
    use crate::domain::events::{CampaignRunStarted, SessionEventKind};
    use otherworlds_test_support::{EmptyEventRepository, RecordingEventRepository};

    #[tokio::test]
    async fn test_get_campaign_run_by_id_returns_view_with_state() {
        // Arrange
        let run_id = Uuid::new_v4();
        let campaign_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        let events = vec![StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: run_id,
            event_type: "session.campaign_run_started".to_owned(),
            payload: serde_json::to_value(SessionEventKind::CampaignRunStarted(
                CampaignRunStarted {
                    run_id,
                    campaign_id,
                },
            ))
            .unwrap(),
            sequence_number: 1,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }];
        let repo = RecordingEventRepository::new(Ok(events));

        // Act
        let view = get_campaign_run_by_id(run_id, &repo).await.unwrap();

        // Assert
        assert_eq!(view.run_id, run_id);
        assert_eq!(view.campaign_id, Some(campaign_id));
        assert!(view.checkpoint_ids.is_empty());
        assert_eq!(view.version, 1);
    }

    #[tokio::test]
    async fn test_get_campaign_run_by_id_returns_not_found_when_no_events() {
        // Arrange
        let run_id = Uuid::new_v4();
        let repo = EmptyEventRepository;

        // Act
        let result = get_campaign_run_by_id(run_id, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, run_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }
}
