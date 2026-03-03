//! Command handlers for the Session & Progress context.
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
use tracing::instrument;
use uuid::Uuid;

use crate::domain::aggregates::CampaignRun;
use crate::domain::commands::{
    ArchiveCampaignRun, BranchTimeline, CreateCheckpoint, StartCampaignRun,
};
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
#[instrument(skip(clock, rng, repo), fields(campaign_id = %command.campaign_id, correlation_id = %command.correlation_id))]
pub async fn handle_start_campaign_run(
    command: &StartCampaignRun,
    clock: &dyn Clock,
    rng: &Mutex<dyn DeterministicRng + Send>,
    repo: &dyn EventRepository,
) -> Result<SessionCommandResult, DomainError> {
    let run_id = {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        rng_guard.next_uuid()
    };
    let mut run = CampaignRun::new(run_id);

    {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        run.start_campaign_run(
            command.campaign_id,
            command.correlation_id,
            clock,
            &mut *rng_guard,
        );
    }

    let stored_events: Vec<StoredEvent> = run
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(run_id, run.version(), &stored_events)
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
#[instrument(skip(clock, rng, repo), fields(run_id = %command.run_id, correlation_id = %command.correlation_id))]
pub async fn handle_create_checkpoint(
    command: &CreateCheckpoint,
    clock: &dyn Clock,
    rng: &Mutex<dyn DeterministicRng + Send>,
    repo: &dyn EventRepository,
) -> Result<SessionCommandResult, DomainError> {
    let existing_events = repo.load_events(command.run_id).await?;
    if existing_events.is_empty() {
        return Err(DomainError::AggregateNotFound(command.run_id));
    }
    let mut run = reconstitute(command.run_id, &existing_events)?;
    if run.archived {
        return Err(DomainError::Validation("campaign run is archived".into()));
    }

    {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        run.create_checkpoint(command.correlation_id, clock, &mut *rng_guard);
    }

    let stored_events: Vec<StoredEvent> = run
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.run_id, run.version(), &stored_events)
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
#[instrument(skip(clock, rng, repo), fields(source_run_id = %command.source_run_id, correlation_id = %command.correlation_id))]
pub async fn handle_branch_timeline(
    command: &BranchTimeline,
    clock: &dyn Clock,
    rng: &Mutex<dyn DeterministicRng + Send>,
    repo: &dyn EventRepository,
) -> Result<SessionCommandResult, DomainError> {
    // Load source run events to verify it exists.
    let source_events = repo.load_events(command.source_run_id).await?;
    if source_events.is_empty() {
        return Err(DomainError::AggregateNotFound(command.source_run_id));
    }
    let source_run = reconstitute(command.source_run_id, &source_events)?;
    if source_run.archived {
        return Err(DomainError::Validation("campaign run is archived".into()));
    }

    let branch_run_id = {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        rng_guard.next_uuid()
    };
    let mut branch = CampaignRun::new(branch_run_id);

    {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        branch.branch_timeline(
            command.source_run_id,
            command.from_checkpoint_id,
            command.correlation_id,
            clock,
            &mut *rng_guard,
        );
    }

    let stored_events: Vec<StoredEvent> = branch
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(branch_run_id, branch.version(), &stored_events)
        .await?;

    Ok(SessionCommandResult {
        aggregate_id: branch_run_id,
        stored_events,
    })
}

/// Handles the `ArchiveCampaignRun` command: reconstitutes the aggregate,
/// archives it (soft-delete), and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError::AggregateNotFound` if no events exist for the run ID.
/// Returns `DomainError::Validation` if the campaign run is already archived.
/// Returns `DomainError` if event loading or appending fails.
#[instrument(skip(clock, rng, repo), fields(run_id = %command.run_id, correlation_id = %command.correlation_id))]
pub async fn handle_archive_campaign_run(
    command: &ArchiveCampaignRun,
    clock: &dyn Clock,
    rng: &Mutex<dyn DeterministicRng + Send>,
    repo: &dyn EventRepository,
) -> Result<SessionCommandResult, DomainError> {
    let existing_events = repo.load_events(command.run_id).await?;
    if existing_events.is_empty() {
        return Err(DomainError::AggregateNotFound(command.run_id));
    }
    let mut run = reconstitute(command.run_id, &existing_events)?;

    {
        let mut rng_guard = rng
            .lock()
            .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
        run.archive(command.correlation_id, clock, &mut *rng_guard)?;
    }

    let stored_events: Vec<StoredEvent> = run
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.run_id, run.version(), &stored_events)
        .await?;

    Ok(SessionCommandResult {
        aggregate_id: command.run_id,
        stored_events,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Utc};
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::StoredEvent;
    use otherworlds_core::rng::DeterministicRng;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    use crate::application::command_handlers::{
        handle_archive_campaign_run, handle_branch_timeline, handle_create_checkpoint,
        handle_start_campaign_run,
    };
    use crate::domain::commands::{
        ArchiveCampaignRun, BranchTimeline, CreateCheckpoint, StartCampaignRun,
    };
    use crate::domain::events::{CampaignRunArchived, CampaignRunStarted, SessionEventKind};
    use otherworlds_test_support::{FixedClock, MockRng, RecordingEventRepository};

    #[tokio::test]
    async fn test_handle_start_campaign_run_persists_campaign_run_started_event() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = StartCampaignRun {
            correlation_id,
            campaign_id,
        };

        // Act
        let result = handle_start_campaign_run(&command, &clock, &*rng, &repo).await;

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
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let existing_event = dummy_stored_event(run_id, fixed_now);
        let repo = RecordingEventRepository::new(Ok(vec![existing_event]));

        let command = CreateCheckpoint {
            correlation_id,
            run_id,
        };

        // Act
        let result = handle_create_checkpoint(&command, &clock, &*rng, &repo).await;

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
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = CreateCheckpoint {
            correlation_id,
            run_id,
        };

        // Act
        let result = handle_create_checkpoint(&command, &clock, &*rng, &repo).await;

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
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let source_event = dummy_stored_event(source_run_id, fixed_now);
        let repo = RecordingEventRepository::new(Ok(vec![source_event]));

        let command = BranchTimeline {
            correlation_id,
            source_run_id,
            from_checkpoint_id,
        };

        // Act
        let result = handle_branch_timeline(&command, &clock, &*rng, &repo).await;

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
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = BranchTimeline {
            correlation_id,
            source_run_id,
            from_checkpoint_id,
        };

        // Act
        let result = handle_branch_timeline(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, source_run_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_archive_campaign_run_persists_campaign_run_archived_event() {
        // Arrange
        let run_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let existing_event = dummy_stored_event(run_id, fixed_now);
        let repo = RecordingEventRepository::new(Ok(vec![existing_event]));

        let command = ArchiveCampaignRun {
            correlation_id,
            run_id,
        };

        // Act
        let result = handle_archive_campaign_run(&command, &clock, &*rng, &repo).await;

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
        assert_eq!(stored.event_type, "session.campaign_run_archived");
        assert_eq!(stored.aggregate_id, run_id);
        assert_eq!(stored.sequence_number, 2);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);
    }

    #[tokio::test]
    async fn test_handle_archive_campaign_run_returns_error_when_run_not_found() {
        // Arrange
        let run_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = ArchiveCampaignRun {
            correlation_id,
            run_id,
        };

        // Act
        let result = handle_archive_campaign_run(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, run_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_archive_campaign_run_returns_error_when_already_archived() {
        // Arrange
        let run_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let existing_event = dummy_stored_event(run_id, fixed_now);
        let archived_event = StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: run_id,
            event_type: "session.campaign_run_archived".to_owned(),
            payload: serde_json::to_value(SessionEventKind::CampaignRunArchived(
                CampaignRunArchived { run_id },
            ))
            .unwrap(),
            sequence_number: 2,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        };
        let repo = RecordingEventRepository::new(Ok(vec![existing_event, archived_event]));

        let command = ArchiveCampaignRun {
            correlation_id,
            run_id,
        };

        // Act
        let result = handle_archive_campaign_run(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "campaign run is already archived");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_create_checkpoint_returns_error_when_archived() {
        // Arrange
        let run_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let existing_event = dummy_stored_event(run_id, fixed_now);
        let archived_event = StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: run_id,
            event_type: "session.campaign_run_archived".to_owned(),
            payload: serde_json::to_value(SessionEventKind::CampaignRunArchived(
                CampaignRunArchived { run_id },
            ))
            .unwrap(),
            sequence_number: 2,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        };
        let repo = RecordingEventRepository::new(Ok(vec![existing_event, archived_event]));

        let command = CreateCheckpoint {
            correlation_id,
            run_id,
        };

        // Act
        let result = handle_create_checkpoint(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "campaign run is archived");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_branch_timeline_returns_error_when_source_archived() {
        // Arrange
        let source_run_id = Uuid::new_v4();
        let from_checkpoint_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let existing_event = dummy_stored_event(source_run_id, fixed_now);
        let archived_event = StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: source_run_id,
            event_type: "session.campaign_run_archived".to_owned(),
            payload: serde_json::to_value(SessionEventKind::CampaignRunArchived(
                CampaignRunArchived {
                    run_id: source_run_id,
                },
            ))
            .unwrap(),
            sequence_number: 2,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        };
        let repo = RecordingEventRepository::new(Ok(vec![existing_event, archived_event]));

        let command = BranchTimeline {
            correlation_id,
            source_run_id,
            from_checkpoint_id,
        };

        // Act
        let result = handle_branch_timeline(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "campaign run is archived");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
