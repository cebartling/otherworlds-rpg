//! Aggregate roots for the Narrative Orchestration context.

use otherworlds_core::aggregate::AggregateRoot;
use otherworlds_core::clock::Clock;
use otherworlds_core::event::EventMetadata;
use uuid::Uuid;

use super::events::{BeatAdvanced, ChoicePresented, NarrativeEvent, NarrativeEventKind};

/// The aggregate root for a narrative session.
#[derive(Debug)]
pub struct NarrativeSession {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub(crate) version: i64,
    /// The most recent beat ID.
    pub(crate) current_beat_id: Option<Uuid>,
    /// All choice IDs presented in this session.
    pub(crate) choice_ids: Vec<Uuid>,
    /// Uncommitted events pending persistence.
    uncommitted_events: Vec<NarrativeEvent>,
}

impl NarrativeSession {
    /// Creates a new narrative session.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            version: 0,
            current_beat_id: None,
            choice_ids: Vec::new(),
            uncommitted_events: Vec::new(),
        }
    }

    /// Returns the next sequence number for a new event.
    #[allow(clippy::cast_possible_wrap)]
    fn next_sequence_number(&self) -> i64 {
        self.version + self.uncommitted_events.len() as i64 + 1
    }

    /// Advances the narrative to the next beat, producing a `BeatAdvanced` event.
    pub fn advance_beat(&mut self, correlation_id: Uuid, clock: &dyn Clock) {
        // TODO: event_id and beat_id use Uuid::new_v4() which breaks replay determinism.
        // Requires extending DeterministicRng to support UUID generation and
        // threading &mut dyn DeterministicRng through all domain methods.
        let event = NarrativeEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "narrative.beat_advanced".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: NarrativeEventKind::BeatAdvanced(BeatAdvanced {
                session_id: self.id,
                beat_id: Uuid::new_v4(),
            }),
        };

        self.uncommitted_events.push(event);
    }

    /// Presents a choice to the player, producing a `ChoicePresented` event.
    pub fn present_choice(&mut self, correlation_id: Uuid, clock: &dyn Clock) {
        // TODO: event_id and choice_id use Uuid::new_v4() which breaks replay determinism.
        // See TODO on `advance_beat()` for details.
        let event = NarrativeEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "narrative.choice_presented".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: NarrativeEventKind::ChoicePresented(ChoicePresented {
                session_id: self.id,
                choice_id: Uuid::new_v4(),
            }),
        };

        self.uncommitted_events.push(event);
    }
}

impl AggregateRoot for NarrativeSession {
    type Event = NarrativeEvent;

    fn aggregate_id(&self) -> Uuid {
        self.id
    }

    fn version(&self) -> i64 {
        self.version
    }

    fn apply(&mut self, event: &Self::Event) {
        match &event.kind {
            NarrativeEventKind::BeatAdvanced(payload) => {
                self.current_beat_id = Some(payload.beat_id);
            }
            NarrativeEventKind::ChoicePresented(payload) => {
                self.choice_ids.push(payload.choice_id);
            }
            NarrativeEventKind::SceneStarted(_) => {}
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
    fn test_advance_beat_produces_beat_advanced_event() {
        // Arrange
        let session_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut session = NarrativeSession::new(session_id);

        // Act
        session.advance_beat(correlation_id, &clock);

        // Assert
        let events = session.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "narrative.beat_advanced");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, session_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            NarrativeEventKind::BeatAdvanced(payload) => {
                assert_eq!(payload.session_id, session_id);
            }
            other => panic!("expected BeatAdvanced, got {other:?}"),
        }
    }

    #[test]
    fn test_present_choice_produces_choice_presented_event() {
        // Arrange
        let session_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut session = NarrativeSession::new(session_id);

        // Act
        session.present_choice(correlation_id, &clock);

        // Assert
        let events = session.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "narrative.choice_presented");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, session_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            NarrativeEventKind::ChoicePresented(payload) => {
                assert_eq!(payload.session_id, session_id);
            }
            other => panic!("expected ChoicePresented, got {other:?}"),
        }
    }
}
