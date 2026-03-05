//! Branch orchestration — coordinates cross-context event replay during timeline branching.

use std::sync::Mutex;

use otherworlds_core::branching::clone_events_for_branch;
use otherworlds_core::clock::Clock;
use otherworlds_core::error::DomainError;
use otherworlds_core::repository::EventRepository;
use otherworlds_core::rng::DeterministicRng;
use tracing::{info, instrument};
use uuid::Uuid;

use otherworlds_session::application::command_handlers as session_handlers;
use otherworlds_session::domain::commands::{BranchTimeline, RegisterAggregate};

/// Result of a cross-context branch operation.
#[derive(Debug)]
pub struct BranchResult {
    /// The new branch run aggregate ID.
    pub branch_run_id: Uuid,
    /// All event IDs produced across all contexts.
    pub event_ids: Vec<Uuid>,
    /// Mapping of `context_name` → `new_aggregate_id` for cloned aggregates.
    pub cloned_aggregates: Vec<(String, Uuid)>,
}

/// Orchestrates a full timeline branch: branches the session context, then
/// clones registered aggregates from other bounded contexts into the fork.
///
/// Steps:
/// 1. Branch the session (replays session events, produces `TimelineBranched`).
/// 2. Load the source run to read its `registered_aggregates`.
/// 3. For each registered aggregate, load source events, clone them with
///    new IDs, and persist as a new aggregate.
/// 4. Register each cloned aggregate with the branched run.
///
/// # Errors
///
/// Returns `DomainError` if any step fails (source not found, archived, etc.).
#[instrument(skip(clock, rng, repo), fields(source_run_id = %source_run_id, from_checkpoint_id = %from_checkpoint_id))]
pub async fn orchestrate_branch_timeline(
    source_run_id: Uuid,
    from_checkpoint_id: Uuid,
    correlation_id: Uuid,
    clock: &(dyn Clock + Send + Sync),
    rng: &Mutex<dyn DeterministicRng + Send>,
    repo: &dyn EventRepository,
) -> Result<BranchResult, DomainError> {
    // Step 1: Branch the session context.
    let branch_command = BranchTimeline {
        correlation_id,
        source_run_id,
        from_checkpoint_id,
    };

    let session_result =
        session_handlers::handle_branch_timeline(&branch_command, clock, rng, repo).await?;
    let branch_run_id = session_result.aggregate_id;

    let mut all_event_ids: Vec<Uuid> = session_result
        .stored_events
        .iter()
        .map(|e| e.event_id)
        .collect();

    info!(branch_run_id = %branch_run_id, "session branch created, cloning cross-context aggregates");

    // Step 2: Load the source run to get registered_aggregates.
    let source_events = repo.load_events(source_run_id).await?;
    let source_run = session_handlers::reconstitute(source_run_id, &source_events)?;

    let mut cloned_aggregates = Vec::new();

    // Step 3: For each registered aggregate, clone its events.
    for (context_name, source_aggregate_id) in source_run.registered_aggregates() {
        let source_agg_events = repo.load_events(*source_aggregate_id).await?;
        if source_agg_events.is_empty() {
            continue;
        }

        let new_aggregate_id = {
            let mut rng_guard = rng
                .lock()
                .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
            rng_guard.next_uuid()
        };

        let cloned_events = {
            let mut rng_guard = rng
                .lock()
                .map_err(|e| DomainError::Infrastructure(format!("RNG mutex poisoned: {e}")))?;
            clone_events_for_branch(
                &source_agg_events,
                *source_aggregate_id,
                new_aggregate_id,
                correlation_id,
                1,
                clock,
                &mut *rng_guard,
            )
        };

        info!(
            context = %context_name,
            source_aggregate_id = %source_aggregate_id,
            new_aggregate_id = %new_aggregate_id,
            event_count = cloned_events.len(),
            "cloned aggregate events for branch"
        );

        all_event_ids.extend(cloned_events.iter().map(|e| e.event_id));
        repo.append_events(new_aggregate_id, 0, &cloned_events)
            .await?;

        // Step 4: Register the cloned aggregate with the branched run.
        let register_command = RegisterAggregate {
            correlation_id,
            run_id: branch_run_id,
            context_name: context_name.clone(),
            aggregate_id: new_aggregate_id,
        };

        let register_result =
            session_handlers::handle_register_aggregate(&register_command, clock, rng, repo)
                .await?;
        all_event_ids.extend(register_result.stored_events.iter().map(|e| e.event_id));

        cloned_aggregates.push((context_name.clone(), new_aggregate_id));
    }

    Ok(BranchResult {
        branch_run_id,
        event_ids: all_event_ids,
        cloned_aggregates,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use chrono::{TimeZone, Utc};
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::StoredEvent;
    use otherworlds_core::rng::DeterministicRng;
    use otherworlds_session::domain::events::{
        AggregateRegistered, CampaignRunStarted, CheckpointCreated, SessionEventKind,
    };
    use otherworlds_test_support::{FixedClock, MockRng, MultiAggregateEventRepository, SequenceRng};
    use uuid::Uuid;

    use super::*;

    /// Creates a `SequenceRng` with enough distinct u32 values for UUID generation.
    fn test_rng() -> SequenceRng {
        SequenceRng::new((1..=200).collect())
    }

    fn make_session_events(
        run_id: Uuid,
        campaign_id: Uuid,
        checkpoint_id: Uuid,
    ) -> Vec<StoredEvent> {
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        vec![
            StoredEvent {
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
            },
            StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: run_id,
                event_type: "session.checkpoint_created".to_owned(),
                payload: serde_json::to_value(SessionEventKind::CheckpointCreated(
                    CheckpointCreated {
                        run_id,
                        checkpoint_id,
                    },
                ))
                .unwrap(),
                sequence_number: 2,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
        ]
    }

    fn make_registered_event(
        run_id: Uuid,
        context_name: &str,
        aggregate_id: Uuid,
        seq: i64,
    ) -> StoredEvent {
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: run_id,
            event_type: "session.aggregate_registered".to_owned(),
            payload: serde_json::to_value(SessionEventKind::AggregateRegistered(
                AggregateRegistered {
                    run_id,
                    context_name: context_name.to_owned(),
                    aggregate_id,
                },
            ))
            .unwrap(),
            sequence_number: seq,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }
    }

    fn make_context_events(aggregate_id: Uuid, event_type: &str, count: usize) -> Vec<StoredEvent> {
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        (0..count)
            .map(|i| {
                #[allow(clippy::cast_possible_wrap)]
                let seq = i as i64 + 1;
                StoredEvent {
                    event_id: Uuid::new_v4(),
                    aggregate_id,
                    event_type: event_type.to_owned(),
                    payload: serde_json::json!({
                        "id": aggregate_id.to_string(),
                        "data": format!("event_{i}"),
                    }),
                    sequence_number: seq,
                    correlation_id: Uuid::new_v4(),
                    causation_id: Uuid::new_v4(),
                    occurred_at: fixed_now,
                }
            })
            .collect()
    }

    #[tokio::test]
    async fn test_orchestrate_branch_without_registered_aggregates() {
        // Arrange — source run with no registered aggregates
        let source_run_id = Uuid::new_v4();
        let campaign_id = Uuid::new_v4();
        let checkpoint_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));

        let mut events_map = HashMap::new();
        events_map.insert(
            source_run_id,
            make_session_events(source_run_id, campaign_id, checkpoint_id),
        );
        let repo = MultiAggregateEventRepository::new(events_map);

        // Act
        let result = orchestrate_branch_timeline(
            source_run_id,
            checkpoint_id,
            correlation_id,
            &clock,
            &*rng,
            &repo,
        )
        .await;

        // Assert
        let branch = result.unwrap();
        assert_ne!(branch.branch_run_id, source_run_id);
        assert!(branch.cloned_aggregates.is_empty());
        // Session events: CampaignRunStarted + CheckpointCreated + TimelineBranched = 3
        assert_eq!(branch.event_ids.len(), 3);
    }

    #[tokio::test]
    async fn test_orchestrate_branch_clones_registered_aggregate() {
        // Arrange — source run with one registered narrative aggregate
        let source_run_id = Uuid::new_v4();
        let campaign_id = Uuid::new_v4();
        let checkpoint_id = Uuid::new_v4();
        let narrative_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(test_rng()));

        let mut session_events = make_session_events(source_run_id, campaign_id, checkpoint_id);
        session_events.push(make_registered_event(
            source_run_id,
            "narrative",
            narrative_id,
            3,
        ));

        let narrative_events = make_context_events(narrative_id, "narrative.beat_advanced", 2);

        let mut events_map = HashMap::new();
        events_map.insert(source_run_id, session_events);
        events_map.insert(narrative_id, narrative_events);
        let repo = MultiAggregateEventRepository::new(events_map);

        // Act
        let result = orchestrate_branch_timeline(
            source_run_id,
            checkpoint_id,
            correlation_id,
            &clock,
            &*rng,
            &repo,
        )
        .await;

        // Assert
        let branch = result.unwrap();
        assert_eq!(branch.cloned_aggregates.len(), 1);
        assert_eq!(branch.cloned_aggregates[0].0, "narrative");
        assert_ne!(branch.cloned_aggregates[0].1, narrative_id);

        // Verify events were appended for the cloned aggregate
        let appended = repo.appended_events();

        // appended[0] = session branch events (3)
        // appended[1] = cloned narrative events (2)
        // appended[2] = register aggregate event (1)
        assert!(appended.len() >= 3);

        // Check cloned narrative events
        let (cloned_agg_id, expected_version, cloned_events) = &appended[1];
        assert_eq!(*cloned_agg_id, branch.cloned_aggregates[0].1);
        assert_eq!(*expected_version, 0);
        assert_eq!(cloned_events.len(), 2);

        // Verify payload IDs were rewritten
        for event in cloned_events {
            assert_eq!(event.aggregate_id, branch.cloned_aggregates[0].1);
            let payload_id = event.payload["id"].as_str().unwrap();
            assert_eq!(payload_id, branch.cloned_aggregates[0].1.to_string());
        }

        // Total event IDs: 3 session + 2 cloned narrative + 1 register = 6
        assert_eq!(branch.event_ids.len(), 6);
    }

    #[tokio::test]
    async fn test_orchestrate_branch_clones_multiple_registered_aggregates() {
        // Arrange — source run with narrative and character registered
        let source_run_id = Uuid::new_v4();
        let campaign_id = Uuid::new_v4();
        let checkpoint_id = Uuid::new_v4();
        let narrative_id = Uuid::new_v4();
        let character_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(test_rng()));

        let mut session_events = make_session_events(source_run_id, campaign_id, checkpoint_id);
        session_events.push(make_registered_event(
            source_run_id,
            "narrative",
            narrative_id,
            3,
        ));
        session_events.push(make_registered_event(
            source_run_id,
            "character",
            character_id,
            4,
        ));

        let narrative_events = make_context_events(narrative_id, "narrative.beat_advanced", 3);
        let character_events = make_context_events(character_id, "character.created", 1);

        let mut events_map = HashMap::new();
        events_map.insert(source_run_id, session_events);
        events_map.insert(narrative_id, narrative_events);
        events_map.insert(character_id, character_events);
        let repo = MultiAggregateEventRepository::new(events_map);

        // Act
        let result = orchestrate_branch_timeline(
            source_run_id,
            checkpoint_id,
            correlation_id,
            &clock,
            &*rng,
            &repo,
        )
        .await;

        // Assert
        let branch = result.unwrap();
        assert_eq!(branch.cloned_aggregates.len(), 2);

        // Total: 3 session + 3 narrative clones + 1 register + 1 character clone + 1 register = 9
        assert_eq!(branch.event_ids.len(), 9);
    }

    #[tokio::test]
    async fn test_orchestrate_branch_returns_error_when_source_not_found() {
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        let repo = MultiAggregateEventRepository::new(HashMap::new());

        let result = orchestrate_branch_timeline(
            Uuid::new_v4(),
            Uuid::new_v4(),
            correlation_id,
            &clock,
            &*rng,
            &repo,
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::AggregateNotFound(_)));
    }
}
