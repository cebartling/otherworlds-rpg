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

/// Event types used by the Session & Progress context.
const EVENT_TYPES: &[&str] = &[
    "session.campaign_run_started",
    "session.checkpoint_created",
    "session.timeline_branched",
];

/// Summary view for listing campaign runs.
#[derive(Debug, Serialize)]
pub struct CampaignRunSummary {
    /// The campaign run identifier.
    pub run_id: Uuid,
    /// The campaign this run belongs to.
    pub campaign_id: Option<Uuid>,
    /// Number of checkpoints created.
    pub checkpoint_count: usize,
    /// Current version (event count).
    pub version: i64,
}

/// Lists all campaign runs.
///
/// # Errors
///
/// Returns `DomainError::Infrastructure` if querying or deserialization fails.
pub async fn list_campaign_runs(
    repo: &dyn EventRepository,
) -> Result<Vec<CampaignRunSummary>, DomainError> {
    let ids = repo.list_aggregate_ids(EVENT_TYPES).await?;
    let mut summaries = Vec::with_capacity(ids.len());
    for id in ids {
        let stored_events = repo.load_events(id).await?;
        if stored_events.is_empty() {
            continue;
        }
        let run = command_handlers::reconstitute(id, &stored_events)?;
        summaries.push(CampaignRunSummary {
            run_id: id,
            campaign_id: run.campaign_id,
            checkpoint_count: run.checkpoint_ids.len(),
            version: run.version,
        });
    }
    Ok(summaries)
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

    use crate::application::query_handlers::{get_campaign_run_by_id, list_campaign_runs};
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

    #[tokio::test]
    async fn test_list_campaign_runs_returns_empty_when_no_aggregates() {
        let repo = EmptyEventRepository;

        let result = list_campaign_runs(&repo).await.unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_list_campaign_runs_returns_summaries() {
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
        let repo = RecordingEventRepository::with_aggregate_ids(Ok(events), vec![run_id]);

        let result = list_campaign_runs(&repo).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].run_id, run_id);
        assert_eq!(result[0].campaign_id, Some(campaign_id));
        assert_eq!(result[0].checkpoint_count, 0);
        assert_eq!(result[0].version, 1);
    }
}
