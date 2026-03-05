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
    ArchiveCampaignRun, BranchTimeline, CreateCheckpoint, RegisterAggregate, StartCampaignRun,
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
pub fn reconstitute(
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

/// Handles the `BranchTimeline` command: loads the source run's events,
/// validates the checkpoint exists, replays source events up to the
/// checkpoint onto a new branch aggregate, and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError::AggregateNotFound` if the source run does not exist.
/// Returns `DomainError::Validation` if the source is archived or the
/// checkpoint ID is not found in the source run.
///
/// # Panics
///
/// Panics if the checkpoint event cannot be found after validation confirms
/// the checkpoint ID exists in the aggregate — this indicates a corrupted
/// event stream and is considered unrecoverable.
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

    // Validate the checkpoint exists in the source run.
    if !source_run
        .checkpoint_ids
        .contains(&command.from_checkpoint_id)
    {
        return Err(DomainError::Validation(format!(
            "checkpoint {} not found in source run",
            command.from_checkpoint_id
        )));
    }

    // Find the index of the CheckpointCreated event for from_checkpoint_id.
    let checkpoint_index = source_events
        .iter()
        .position(|stored| {
            if stored.event_type != "session.checkpoint_created" {
                return false;
            }
            serde_json::from_value::<SessionEventKind>(stored.payload.clone())
                .ok()
                .is_some_and(|kind| {
                    matches!(kind, SessionEventKind::CheckpointCreated(cp) if cp.checkpoint_id == command.from_checkpoint_id)
                })
        })
        .expect("checkpoint validated above, event must exist");

    // Reconstitute source events up to and including the checkpoint as domain events.
    let events_to_replay: Vec<SessionEvent> = source_events[..=checkpoint_index]
        .iter()
        .map(|stored| {
            let kind: SessionEventKind =
                serde_json::from_value(stored.payload.clone()).expect("already validated");
            SessionEvent {
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
            }
        })
        .collect();

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

        // Replay source events (rewrites run_id to branch) up to the checkpoint.
        branch.replay_source_events(
            &events_to_replay,
            command.correlation_id,
            clock,
            &mut *rng_guard,
        );

        // Produce the TimelineBranched event marking the fork point.
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

/// Handles the `RegisterAggregate` command: reconstitutes the aggregate,
/// registers a cross-context aggregate, and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError::AggregateNotFound` if no events exist for the run ID.
/// Returns `DomainError::Validation` if the run is archived or context name is empty.
#[instrument(skip(clock, rng, repo), fields(run_id = %command.run_id, context_name = %command.context_name, correlation_id = %command.correlation_id))]
pub async fn handle_register_aggregate(
    command: &RegisterAggregate,
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
        run.register_aggregate(
            &command.context_name,
            command.aggregate_id,
            command.correlation_id,
            clock,
            &mut *rng_guard,
        )?;
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
        handle_register_aggregate, handle_start_campaign_run,
    };
    use crate::domain::commands::{
        ArchiveCampaignRun, BranchTimeline, CreateCheckpoint, RegisterAggregate, StartCampaignRun,
    };
    use crate::domain::events::{
        CampaignRunArchived, CampaignRunStarted, CheckpointCreated, SessionEventKind,
    };
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
        // Arrange — source has CampaignRunStarted + one CheckpointCreated
        let source_run_id = Uuid::new_v4();
        let from_checkpoint_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let source_events = vec![
            dummy_stored_event(source_run_id, fixed_now),
            StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: source_run_id,
                event_type: "session.checkpoint_created".to_owned(),
                payload: serde_json::to_value(SessionEventKind::CheckpointCreated(
                    CheckpointCreated {
                        run_id: source_run_id,
                        checkpoint_id: from_checkpoint_id,
                    },
                ))
                .unwrap(),
                sequence_number: 2,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
        ];
        let repo = RecordingEventRepository::new(Ok(source_events));

        let command = BranchTimeline {
            correlation_id,
            source_run_id,
            from_checkpoint_id,
        };

        // Act
        let result = handle_branch_timeline(&command, &clock, &*rng, &repo).await;

        // Assert — replayed CampaignRunStarted + CheckpointCreated + TimelineBranched = 3 events
        let cmd_result = result.unwrap();
        assert_eq!(cmd_result.stored_events.len(), 3);

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, cmd_result.aggregate_id);
        assert_eq!(*expected_version, 0);
        assert_eq!(events.len(), 3);

        // Last event should be TimelineBranched
        let last = &events[2];
        assert_eq!(last.event_type, "session.timeline_branched");
        assert_eq!(last.aggregate_id, cmd_result.aggregate_id);
        assert_eq!(last.sequence_number, 3);
        assert_eq!(last.correlation_id, correlation_id);
        assert_eq!(last.causation_id, correlation_id);
        assert_eq!(last.occurred_at, fixed_now);
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

    /// Helper: builds source run events with a `CampaignRunStarted` + two `CheckpointCreated` events.
    fn source_run_with_checkpoints(
        source_run_id: Uuid,
        campaign_id: Uuid,
        checkpoint_1_id: Uuid,
        checkpoint_2_id: Uuid,
        fixed_now: DateTime<Utc>,
    ) -> Vec<StoredEvent> {
        vec![
            StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: source_run_id,
                event_type: "session.campaign_run_started".to_owned(),
                payload: serde_json::to_value(SessionEventKind::CampaignRunStarted(
                    CampaignRunStarted {
                        run_id: source_run_id,
                        campaign_id,
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
                aggregate_id: source_run_id,
                event_type: "session.checkpoint_created".to_owned(),
                payload: serde_json::to_value(SessionEventKind::CheckpointCreated(
                    CheckpointCreated {
                        run_id: source_run_id,
                        checkpoint_id: checkpoint_1_id,
                    },
                ))
                .unwrap(),
                sequence_number: 2,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: source_run_id,
                event_type: "session.checkpoint_created".to_owned(),
                payload: serde_json::to_value(SessionEventKind::CheckpointCreated(
                    CheckpointCreated {
                        run_id: source_run_id,
                        checkpoint_id: checkpoint_2_id,
                    },
                ))
                .unwrap(),
                sequence_number: 3,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
        ]
    }

    #[tokio::test]
    async fn test_handle_branch_timeline_replays_source_events_up_to_checkpoint() {
        // Arrange — source run with 2 checkpoints, branch from the first
        let source_run_id = Uuid::new_v4();
        let campaign_id = Uuid::new_v4();
        let checkpoint_1_id = Uuid::new_v4();
        let checkpoint_2_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let source_events = source_run_with_checkpoints(
            source_run_id,
            campaign_id,
            checkpoint_1_id,
            checkpoint_2_id,
            fixed_now,
        );
        let repo = RecordingEventRepository::new(Ok(source_events));

        let command = BranchTimeline {
            correlation_id,
            source_run_id,
            from_checkpoint_id: checkpoint_1_id,
        };

        // Act
        let result = handle_branch_timeline(&command, &clock, &*rng, &repo).await;

        // Assert
        let cmd_result = result.unwrap();
        let branch_run_id = cmd_result.aggregate_id;

        // Should have: CampaignRunStarted + CheckpointCreated(cp1) + TimelineBranched = 3 events
        assert_eq!(cmd_result.stored_events.len(), 3);

        // Event 1: replayed CampaignRunStarted with branch's aggregate_id
        let ev1 = &cmd_result.stored_events[0];
        assert_eq!(ev1.event_type, "session.campaign_run_started");
        assert_eq!(ev1.aggregate_id, branch_run_id);
        assert_eq!(ev1.sequence_number, 1);
        let kind1: SessionEventKind = serde_json::from_value(ev1.payload.clone()).unwrap();
        match kind1 {
            SessionEventKind::CampaignRunStarted(payload) => {
                assert_eq!(payload.run_id, branch_run_id);
                assert_eq!(payload.campaign_id, campaign_id);
            }
            other => panic!("expected CampaignRunStarted, got {other:?}"),
        }

        // Event 2: replayed CheckpointCreated(cp1) — preserves checkpoint_id
        let ev2 = &cmd_result.stored_events[1];
        assert_eq!(ev2.event_type, "session.checkpoint_created");
        assert_eq!(ev2.aggregate_id, branch_run_id);
        assert_eq!(ev2.sequence_number, 2);
        let kind2: SessionEventKind = serde_json::from_value(ev2.payload.clone()).unwrap();
        match kind2 {
            SessionEventKind::CheckpointCreated(payload) => {
                assert_eq!(payload.run_id, branch_run_id);
                assert_eq!(payload.checkpoint_id, checkpoint_1_id);
            }
            other => panic!("expected CheckpointCreated, got {other:?}"),
        }

        // Event 3: TimelineBranched
        let ev3 = &cmd_result.stored_events[2];
        assert_eq!(ev3.event_type, "session.timeline_branched");
        assert_eq!(ev3.aggregate_id, branch_run_id);
        assert_eq!(ev3.sequence_number, 3);
    }

    #[tokio::test]
    async fn test_handle_branch_timeline_rejects_invalid_checkpoint_id() {
        // Arrange — source run with a checkpoint, but branch from a non-existent checkpoint
        let source_run_id = Uuid::new_v4();
        let campaign_id = Uuid::new_v4();
        let real_checkpoint_id = Uuid::new_v4();
        let bogus_checkpoint_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let source_events = source_run_with_checkpoints(
            source_run_id,
            campaign_id,
            real_checkpoint_id,
            Uuid::new_v4(),
            fixed_now,
        );
        let repo = RecordingEventRepository::new(Ok(source_events));

        let command = BranchTimeline {
            correlation_id,
            source_run_id,
            from_checkpoint_id: bogus_checkpoint_id,
        };

        // Act
        let result = handle_branch_timeline(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert!(
                    msg.contains("checkpoint"),
                    "expected checkpoint validation error, got: {msg}"
                );
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_register_aggregate_persists_aggregate_registered_event() {
        // Arrange
        let run_id = Uuid::new_v4();
        let narrative_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let existing_event = dummy_stored_event(run_id, fixed_now);
        let repo = RecordingEventRepository::new(Ok(vec![existing_event]));

        let command = RegisterAggregate {
            correlation_id,
            run_id,
            context_name: "narrative".to_owned(),
            aggregate_id: narrative_id,
        };

        // Act
        let result = handle_register_aggregate(&command, &clock, &*rng, &repo).await;

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
        assert_eq!(stored.event_type, "session.aggregate_registered");
        assert_eq!(stored.aggregate_id, run_id);
        assert_eq!(stored.sequence_number, 2);
        assert_eq!(stored.correlation_id, correlation_id);
    }

    #[tokio::test]
    async fn test_handle_register_aggregate_returns_error_when_run_not_found() {
        // Arrange
        let run_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = RegisterAggregate {
            correlation_id,
            run_id,
            context_name: "narrative".to_owned(),
            aggregate_id: Uuid::new_v4(),
        };

        // Act
        let result = handle_register_aggregate(&command, &clock, &*rng, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, run_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }
}
