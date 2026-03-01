//! Aggregate roots for the Rules & Resolution context.

use otherworlds_core::aggregate::AggregateRoot;
use otherworlds_core::clock::Clock;
use otherworlds_core::event::EventMetadata;
use otherworlds_core::rng::DeterministicRng;
use uuid::Uuid;

use super::events::{CheckPerformed, IntentResolved, RulesEvent, RulesEventKind};

/// The aggregate root for a resolution.
#[derive(Debug)]
pub struct Resolution {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub(crate) version: i64,
    /// Intent IDs resolved during this resolution.
    pub(crate) intent_ids: Vec<Uuid>,
    /// Check IDs performed during this resolution.
    pub(crate) check_ids: Vec<Uuid>,
    /// Uncommitted events pending persistence.
    uncommitted_events: Vec<RulesEvent>,
}

impl Resolution {
    /// Creates a new resolution.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            version: 0,
            intent_ids: Vec::new(),
            check_ids: Vec::new(),
            uncommitted_events: Vec::new(),
        }
    }

    /// Returns the next sequence number for a new event.
    #[allow(clippy::cast_possible_wrap)]
    fn next_sequence_number(&self) -> i64 {
        self.version + self.uncommitted_events.len() as i64 + 1
    }

    /// Resolves a player intent, producing an `IntentResolved` event.
    pub fn resolve_intent(&mut self, intent_id: Uuid, correlation_id: Uuid, clock: &dyn Clock) {
        // TODO: event_id uses Uuid::new_v4() which breaks replay determinism.
        // Requires extending DeterministicRng to support UUID generation and
        // threading &mut dyn DeterministicRng through all domain methods.
        let event = RulesEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "rules.intent_resolved".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: RulesEventKind::IntentResolved(IntentResolved {
                resolution_id: self.id,
                intent_id,
            }),
        };

        self.uncommitted_events.push(event);
    }

    /// Performs a check (skill, combat, etc.), producing a `CheckPerformed` event.
    ///
    /// Uses `DeterministicRng` for generating the check identifier, ensuring
    /// deterministic replay when a seeded RNG is provided.
    pub fn perform_check(
        &mut self,
        correlation_id: Uuid,
        clock: &dyn Clock,
        rng: &mut dyn DeterministicRng,
    ) {
        // TODO: event_id uses Uuid::new_v4() which breaks replay determinism.
        // See TODO on `resolve_intent()` for details.

        // Generate a deterministic check_id from two RNG-produced u32 halves.
        let hi = u64::from(rng.next_u32_range(0, u32::MAX));
        let lo = u64::from(rng.next_u32_range(0, u32::MAX));
        let bits = (hi << 32) | lo;
        let check_id = Uuid::from_u64_pair(bits, bits.wrapping_mul(0x517c_c1b7_2722_0a95));

        let event = RulesEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "rules.check_performed".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: RulesEventKind::CheckPerformed(CheckPerformed {
                resolution_id: self.id,
                check_id,
            }),
        };

        self.uncommitted_events.push(event);
    }
}

impl AggregateRoot for Resolution {
    type Event = RulesEvent;

    fn aggregate_id(&self) -> Uuid {
        self.id
    }

    fn version(&self) -> i64 {
        self.version
    }

    fn apply(&mut self, event: &Self::Event) {
        match &event.kind {
            RulesEventKind::IntentResolved(payload) => {
                self.intent_ids.push(payload.intent_id);
            }
            RulesEventKind::CheckPerformed(payload) => {
                self.check_ids.push(payload.check_id);
            }
            RulesEventKind::EffectsProduced(_) => {}
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
    use otherworlds_test_support::{FixedClock, SequenceRng};

    #[test]
    fn test_resolve_intent_produces_intent_resolved_event() {
        // Arrange
        let resolution_id = Uuid::new_v4();
        let intent_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut resolution = Resolution::new(resolution_id);

        // Act
        resolution.resolve_intent(intent_id, correlation_id, &clock);

        // Assert
        let events = resolution.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "rules.intent_resolved");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, resolution_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            RulesEventKind::IntentResolved(payload) => {
                assert_eq!(payload.resolution_id, resolution_id);
                assert_eq!(payload.intent_id, intent_id);
            }
            other => panic!("expected IntentResolved, got {other:?}"),
        }
    }

    #[test]
    fn test_apply_intent_resolved_pushes_intent_id() {
        // Arrange
        let resolution_id = Uuid::new_v4();
        let intent_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let mut resolution = Resolution::new(resolution_id);
        let event = RulesEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "rules.intent_resolved".to_owned(),
                aggregate_id: resolution_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            kind: RulesEventKind::IntentResolved(IntentResolved {
                resolution_id,
                intent_id,
            }),
        };

        // Act
        resolution.apply(&event);

        // Assert
        assert_eq!(resolution.intent_ids, vec![intent_id]);
        assert_eq!(resolution.version, 1);
    }

    #[test]
    fn test_apply_check_performed_pushes_check_id() {
        // Arrange
        let resolution_id = Uuid::new_v4();
        let check_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let mut resolution = Resolution::new(resolution_id);
        let event = RulesEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "rules.check_performed".to_owned(),
                aggregate_id: resolution_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            kind: RulesEventKind::CheckPerformed(CheckPerformed {
                resolution_id,
                check_id,
            }),
        };

        // Act
        resolution.apply(&event);

        // Assert
        assert_eq!(resolution.check_ids, vec![check_id]);
        assert_eq!(resolution.version, 1);
    }

    #[test]
    fn test_perform_check_produces_check_performed_event() {
        // Arrange
        let resolution_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut resolution = Resolution::new(resolution_id);
        let mut rng = SequenceRng::new(vec![42, 99]);

        // Act
        resolution.perform_check(correlation_id, &clock, &mut rng);

        // Assert
        let events = resolution.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "rules.check_performed");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, resolution_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            RulesEventKind::CheckPerformed(payload) => {
                assert_eq!(payload.resolution_id, resolution_id);
                // check_id should be deterministic given the same RNG seed values.
                assert_ne!(payload.check_id, Uuid::nil());
            }
            other => panic!("expected CheckPerformed, got {other:?}"),
        }
    }
}
