//! Aggregate roots for the Session & Progress context.

use otherworlds_core::aggregate::AggregateRoot;
use otherworlds_core::clock::Clock;
use otherworlds_core::event::EventMetadata;
use uuid::Uuid;

use super::events::{
    CAMPAIGN_RUN_STARTED_EVENT_TYPE, CHECKPOINT_CREATED_EVENT_TYPE, CampaignRunStarted,
    CheckpointCreated, SessionEvent, SessionEventKind, TIMELINE_BRANCHED_EVENT_TYPE,
    TimelineBranched,
};

/// The aggregate root for a campaign run.
#[derive(Debug)]
pub struct CampaignRun {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub(crate) version: i64,
    /// The campaign this run belongs to.
    pub(crate) campaign_id: Option<Uuid>,
    /// Checkpoint IDs created during this run.
    pub(crate) checkpoint_ids: Vec<Uuid>,
    /// Branch source: (`source_run_id`, `from_checkpoint_id`) if this run was branched.
    pub(crate) branch_source: Option<(Uuid, Uuid)>,
    /// Uncommitted events pending persistence.
    uncommitted_events: Vec<SessionEvent>,
}

impl CampaignRun {
    /// Creates a new campaign run.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            version: 0,
            campaign_id: None,
            checkpoint_ids: Vec::new(),
            branch_source: None,
            uncommitted_events: Vec::new(),
        }
    }

    /// Returns the next sequence number for a new event.
    #[allow(clippy::cast_possible_wrap)]
    fn next_sequence_number(&self) -> i64 {
        self.version + self.uncommitted_events.len() as i64 + 1
    }

    /// Starts a campaign run, producing a `CampaignRunStarted` event.
    pub fn start_campaign_run(
        &mut self,
        campaign_id: Uuid,
        correlation_id: Uuid,
        clock: &dyn Clock,
    ) {
        // TODO: event_id uses Uuid::new_v4() which breaks replay determinism.
        // Requires extending DeterministicRng to support UUID generation and
        // threading &mut dyn DeterministicRng through all domain methods.
        let event = SessionEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: CAMPAIGN_RUN_STARTED_EVENT_TYPE.to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: SessionEventKind::CampaignRunStarted(CampaignRunStarted {
                run_id: self.id,
                campaign_id,
            }),
        };

        self.uncommitted_events.push(event);
    }

    /// Creates a checkpoint, producing a `CheckpointCreated` event.
    pub fn create_checkpoint(&mut self, correlation_id: Uuid, clock: &dyn Clock) {
        // TODO: event_id and checkpoint_id use Uuid::new_v4() which breaks replay determinism.
        // See TODO on `start_campaign_run()` for details.
        let event = SessionEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: CHECKPOINT_CREATED_EVENT_TYPE.to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: SessionEventKind::CheckpointCreated(CheckpointCreated {
                run_id: self.id,
                checkpoint_id: Uuid::new_v4(),
            }),
        };

        self.uncommitted_events.push(event);
    }

    /// Branches a timeline, producing a `TimelineBranched` event.
    pub fn branch_timeline(
        &mut self,
        source_run_id: Uuid,
        from_checkpoint_id: Uuid,
        correlation_id: Uuid,
        clock: &dyn Clock,
    ) {
        // TODO: event_id uses Uuid::new_v4() which breaks replay determinism.
        // See TODO on `start_campaign_run()` for details.
        let event = SessionEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: TIMELINE_BRANCHED_EVENT_TYPE.to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: SessionEventKind::TimelineBranched(TimelineBranched {
                source_run_id,
                branch_run_id: self.id,
                from_checkpoint_id,
            }),
        };

        self.uncommitted_events.push(event);
    }
}

impl AggregateRoot for CampaignRun {
    type Event = SessionEvent;

    fn aggregate_id(&self) -> Uuid {
        self.id
    }

    fn version(&self) -> i64 {
        self.version
    }

    fn apply(&mut self, event: &Self::Event) {
        match &event.kind {
            SessionEventKind::CampaignRunStarted(payload) => {
                self.campaign_id = Some(payload.campaign_id);
            }
            SessionEventKind::CheckpointCreated(payload) => {
                self.checkpoint_ids.push(payload.checkpoint_id);
            }
            SessionEventKind::TimelineBranched(payload) => {
                self.branch_source = Some((payload.source_run_id, payload.from_checkpoint_id));
            }
        }
        self.version += 1;
    }

    fn uncommitted_events(&self) -> &[Self::Event] {
        &self.uncommitted_events
    }

    fn clear_uncommitted_events(&mut self) {
        self.uncommitted_events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use otherworlds_core::aggregate::AggregateRoot;
    use otherworlds_core::event::DomainEvent;
    use otherworlds_test_support::FixedClock;

    #[test]
    fn test_start_campaign_run_produces_campaign_run_started_event() {
        // Arrange
        let run_id = Uuid::new_v4();
        let campaign_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut run = CampaignRun::new(run_id);

        // Act
        run.start_campaign_run(campaign_id, correlation_id, &clock);

        // Assert
        let events = run.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "session.campaign_run_started");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, run_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            SessionEventKind::CampaignRunStarted(payload) => {
                assert_eq!(payload.run_id, run_id);
                assert_eq!(payload.campaign_id, campaign_id);
            }
            other => panic!("expected CampaignRunStarted, got {other:?}"),
        }
    }

    #[test]
    fn test_create_checkpoint_produces_checkpoint_created_event() {
        // Arrange
        let run_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut run = CampaignRun::new(run_id);

        // Act
        run.create_checkpoint(correlation_id, &clock);

        // Assert
        let events = run.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "session.checkpoint_created");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, run_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            SessionEventKind::CheckpointCreated(payload) => {
                assert_eq!(payload.run_id, run_id);
            }
            other => panic!("expected CheckpointCreated, got {other:?}"),
        }
    }

    #[test]
    fn test_branch_timeline_produces_timeline_branched_event() {
        // Arrange
        let branch_run_id = Uuid::new_v4();
        let source_run_id = Uuid::new_v4();
        let from_checkpoint_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut run = CampaignRun::new(branch_run_id);

        // Act
        run.branch_timeline(source_run_id, from_checkpoint_id, correlation_id, &clock);

        // Assert
        let events = run.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "session.timeline_branched");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, branch_run_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            SessionEventKind::TimelineBranched(payload) => {
                assert_eq!(payload.source_run_id, source_run_id);
                assert_eq!(payload.branch_run_id, branch_run_id);
                assert_eq!(payload.from_checkpoint_id, from_checkpoint_id);
            }
            other => panic!("expected TimelineBranched, got {other:?}"),
        }
    }
}
