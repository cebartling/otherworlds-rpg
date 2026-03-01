//! Command handlers for the Session & Progress context.
//!
//! This module contains application-level command handler functions that
//! orchestrate domain logic: load aggregate, execute command, persist events.

use otherworlds_core::aggregate::AggregateRoot;
use otherworlds_core::clock::Clock;
use otherworlds_core::error::DomainError;
use otherworlds_core::event::DomainEvent;
use otherworlds_core::repository::{EventRepository, StoredEvent};
use uuid::Uuid;

use crate::domain::aggregates::CampaignRun;
use crate::domain::commands::{BranchTimeline, CreateCheckpoint, StartCampaignRun};
use crate::domain::events::{SessionEvent, SessionEventKind};

/// Result of a successfully handled command.
#[derive(Debug)]
pub struct SessionCommandResult {
    /// The aggregate ID affected or created by the command.
    pub aggregate_id: Uuid,
    /// The stored events produced and persisted.
    pub stored_events: Vec<StoredEvent>,
}

fn to_stored_event(event: &SessionEvent) -> StoredEvent {
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

/// Reconstitutes a `CampaignRun` from stored events.
///
/// # Errors
///
/// Returns `DomainError::Infrastructure` if event deserialization fails.
pub(crate) fn reconstitute(
    run_id: Uuid,
    existing_events: &[StoredEvent],
) -> Result<CampaignRun, DomainError> {
    let mut run = CampaignRun::new(run_id);
    for stored in existing_events {
        let kind: SessionEventKind =
            serde_json::from_value(stored.payload.clone()).map_err(|e| {
                DomainError::Infrastructure(format!("event deserialization failed: {e}"))
            })?;
        let event = SessionEvent {
            metadata: otherworlds_core::event::EventMetadata {
                event_id: stored.event_id,
                event_type: stored.event_type.clone(),
                aggregate_id: stored.aggregate_id,
                sequence_number: stored.sequence_number,
                correlation_id: stored.correlation_id,
                causation_id: stored.causation_id,
                occurred_at: stored.occurred_at,
            },
            kind,
        };
        run.apply(&event);
    }
    Ok(run)
}

/// Handles the `StartCampaignRun` command: creates a new aggregate, starts the
/// campaign run, and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if event appending fails.
pub async fn handle_start_campaign_run(
    command: &StartCampaignRun,
    clock: &dyn Clock,
    repo: &dyn EventRepository,
) -> Result<SessionCommandResult, DomainError> {
    let run_id = Uuid::new_v4();
    let mut run = CampaignRun::new(run_id);

    run.start_campaign_run(command.campaign_id, command.correlation_id, clock);

    let stored_events: Vec<StoredEvent> = run
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(run_id, run.version, &stored_events)
        .await?;

    Ok(SessionCommandResult {
        aggregate_id: run_id,
        stored_events,
    })
}

/// Handles the `CreateCheckpoint` command: reconstitutes the aggregate, creates
/// a checkpoint, and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if event loading or appending fails.
pub async fn handle_create_checkpoint(
    command: &CreateCheckpoint,
    clock: &dyn Clock,
    repo: &dyn EventRepository,
) -> Result<SessionCommandResult, DomainError> {
    let existing_events = repo.load_events(command.run_id).await?;
    if existing_events.is_empty() {
        return Err(DomainError::AggregateNotFound(command.run_id));
    }
    let mut run = reconstitute(command.run_id, &existing_events)?;

    run.create_checkpoint(command.correlation_id, clock);

    let stored_events: Vec<StoredEvent> = run
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.run_id, run.version, &stored_events)
        .await?;

    Ok(SessionCommandResult {
        aggregate_id: command.run_id,
        stored_events,
    })
}

/// Handles the `BranchTimeline` command: loads the source run's events for
/// existence verification, creates a new aggregate for the branch, and
/// persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if event loading or appending fails.
pub async fn handle_branch_timeline(
    command: &BranchTimeline,
    clock: &dyn Clock,
    repo: &dyn EventRepository,
) -> Result<SessionCommandResult, DomainError> {
    // Load source run events to verify it exists.
    let source_events = repo.load_events(command.source_run_id).await?;
    if source_events.is_empty() {
        return Err(DomainError::AggregateNotFound(command.source_run_id));
    }

    let branch_run_id = Uuid::new_v4();
    let mut branch = CampaignRun::new(branch_run_id);

    branch.branch_timeline(
        command.source_run_id,
        command.from_checkpoint_id,
        command.correlation_id,
        clock,
    );

    let stored_events: Vec<StoredEvent> = branch
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(branch_run_id, branch.version, &stored_events)
        .await?;

    Ok(SessionCommandResult {
        aggregate_id: branch_run_id,
        stored_events,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Utc};
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::StoredEvent;
    use uuid::Uuid;

    use crate::application::command_handlers::{
        handle_branch_timeline, handle_create_checkpoint, handle_start_campaign_run,
    };
    use crate::domain::commands::{BranchTimeline, CreateCheckpoint, StartCampaignRun};
    use crate::domain::events::{CampaignRunStarted, SessionEventKind};
    use otherworlds_test_support::{FixedClock, RecordingEventRepository};

    #[tokio::test]
    async fn test_handle_start_campaign_run_persists_campaign_run_started_event() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = StartCampaignRun {
            correlation_id,
            campaign_id,
        };

        // Act
        let result = handle_start_campaign_run(&command, &clock, &repo).await;

        // Assert
        let cmd_result = result.unwrap();
        assert_eq!(cmd_result.stored_events.len(), 1);

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, cmd_result.aggregate_id);
        assert_eq!(*expected_version, 0);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "session.campaign_run_started");
        assert_eq!(stored.aggregate_id, cmd_result.aggregate_id);
        assert_eq!(stored.sequence_number, 1);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);
    }

    #[tokio::test]
    async fn test_handle_create_checkpoint_persists_checkpoint_created_event() {
        // Arrange
        let run_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let existing_event = dummy_stored_event(run_id, fixed_now);
        let repo = RecordingEventRepository::new(Ok(vec![existing_event]));

        let command = CreateCheckpoint {
            correlation_id,
            run_id,
        };

        // Act
        let result = handle_create_checkpoint(&command, &clock, &repo).await;

        // Assert
        let cmd_result = result.unwrap();
        assert_eq!(cmd_result.aggregate_id, run_id);
        assert_eq!(cmd_result.stored_events.len(), 1);

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, run_id);
        assert_eq!(*expected_version, 1);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "session.checkpoint_created");
        assert_eq!(stored.aggregate_id, run_id);
        assert_eq!(stored.sequence_number, 2);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);
    }

    #[tokio::test]
    async fn test_handle_create_checkpoint_returns_error_when_run_not_found() {
        // Arrange
        let run_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = CreateCheckpoint {
            correlation_id,
            run_id,
        };

        // Act
        let result = handle_create_checkpoint(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, run_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }

    fn dummy_stored_event(aggregate_id: Uuid, fixed_now: DateTime<Utc>) -> StoredEvent {
        StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            event_type: "session.campaign_run_started".to_owned(),
            payload: serde_json::to_value(SessionEventKind::CampaignRunStarted(
                CampaignRunStarted {
                    run_id: aggregate_id,
                    campaign_id: Uuid::new_v4(),
                },
            ))
            .unwrap(),
            sequence_number: 1,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }
    }

    #[tokio::test]
    async fn test_handle_branch_timeline_persists_timeline_branched_event() {
        // Arrange
        let source_run_id = Uuid::new_v4();
        let from_checkpoint_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let source_event = dummy_stored_event(source_run_id, fixed_now);
        let repo = RecordingEventRepository::new(Ok(vec![source_event]));

        let command = BranchTimeline {
            correlation_id,
            source_run_id,
            from_checkpoint_id,
        };

        // Act
        let result = handle_branch_timeline(&command, &clock, &repo).await;

        // Assert
        let cmd_result = result.unwrap();
        assert_eq!(cmd_result.stored_events.len(), 1);

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, cmd_result.aggregate_id);
        assert_eq!(*expected_version, 0);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, "session.timeline_branched");
        assert_eq!(stored.aggregate_id, cmd_result.aggregate_id);
        assert_eq!(stored.sequence_number, 1);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);
    }

    #[tokio::test]
    async fn test_handle_branch_timeline_returns_error_when_source_run_not_found() {
        // Arrange
        let source_run_id = Uuid::new_v4();
        let from_checkpoint_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = BranchTimeline {
            correlation_id,
            source_run_id,
            from_checkpoint_id,
        };

        // Act
        let result = handle_branch_timeline(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, source_run_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }
}
