//! Aggregate roots for the Rules & Resolution context.

use otherworlds_core::aggregate::AggregateRoot;
use otherworlds_core::clock::Clock;
use otherworlds_core::error::DomainError;
use otherworlds_core::event::EventMetadata;
use otherworlds_core::rng::DeterministicRng;
use uuid::Uuid;

use super::events::{
    CheckOutcome, CheckResolved, EffectsProduced, IntentDeclared, ResolvedEffect, RulesEvent,
    RulesEventKind, determine_outcome,
};

/// Resolution phase state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResolutionPhase {
    Created,
    IntentDeclared,
    CheckResolved,
    EffectsProduced,
}

/// Captured intent details within the aggregate.
#[derive(Debug, Clone)]
pub(crate) struct DeclaredIntent {
    pub intent_id: Uuid,
    pub action_type: String,
    pub skill: Option<String>,
    pub target_id: Option<Uuid>,
    pub difficulty_class: i32,
    pub modifier: i32,
}

/// Captured check result within the aggregate.
#[derive(Debug, Clone)]
pub(crate) struct CheckResult {
    pub check_id: Uuid,
    pub natural_roll: u32,
    pub modifier: i32,
    pub total: i32,
    pub difficulty_class: i32,
    pub outcome: CheckOutcome,
}

/// The aggregate root for a resolution.
#[derive(Debug)]
pub struct Resolution {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub(crate) version: i64,
    /// Current resolution phase.
    pub(crate) phase: ResolutionPhase,
    /// Declared intent (set after `IntentDeclared`).
    pub(crate) intent: Option<DeclaredIntent>,
    /// Check result (set after `CheckResolved`).
    pub(crate) check_result: Option<CheckResult>,
    /// Produced effects (set after `EffectsProduced`).
    pub(crate) effects: Vec<ResolvedEffect>,
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
            phase: ResolutionPhase::Created,
            intent: None,
            check_result: None,
            effects: Vec::new(),
            uncommitted_events: Vec::new(),
        }
    }

    /// Returns the next sequence number for a new event.
    #[allow(clippy::cast_possible_wrap)]
    fn next_sequence_number(&self) -> i64 {
        self.version + self.uncommitted_events.len() as i64 + 1
    }

    /// Declares a player intent, producing an `IntentDeclared` event.
    ///
    /// # Errors
    ///
    /// Returns `DomainError::Validation` if not in `Created` phase.
    #[allow(clippy::too_many_arguments)]
    pub fn declare_intent(
        &mut self,
        intent_id: Uuid,
        action_type: String,
        skill: Option<String>,
        target_id: Option<Uuid>,
        difficulty_class: i32,
        modifier: i32,
        correlation_id: Uuid,
        clock: &dyn Clock,
    ) -> Result<(), DomainError> {
        if self.phase != ResolutionPhase::Created {
            return Err(DomainError::Validation(
                "resolution must be in Created phase".to_owned(),
            ));
        }

        let event = RulesEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "rules.intent_declared".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: RulesEventKind::IntentDeclared(IntentDeclared {
                resolution_id: self.id,
                intent_id,
                action_type,
                skill,
                target_id,
                difficulty_class,
                modifier,
            }),
        };

        self.uncommitted_events.push(event);
        Ok(())
    }

    /// Resolves a d20 check, producing a `CheckResolved` event.
    ///
    /// Rolls 1d20 using the provided RNG, adds the intent's modifier,
    /// and determines the five-tier outcome.
    ///
    /// # Errors
    ///
    /// Returns `DomainError::Validation` if not in `IntentDeclared` phase.
    ///
    /// # Panics
    ///
    /// Panics if called without a prior `IntentDeclared` event (invariant
    /// guaranteed by phase validation).
    #[allow(clippy::cast_possible_wrap)]
    pub fn resolve_check(
        &mut self,
        correlation_id: Uuid,
        clock: &dyn Clock,
        rng: &mut dyn DeterministicRng,
    ) -> Result<(), DomainError> {
        if self.phase != ResolutionPhase::IntentDeclared {
            return Err(DomainError::Validation(
                "resolution must be in IntentDeclared phase".to_owned(),
            ));
        }

        let intent = self
            .intent
            .as_ref()
            .expect("intent must be set in IntentDeclared phase");

        let natural_roll = rng.next_u32_range(1, 20);
        let total = natural_roll as i32 + intent.modifier;
        let outcome = determine_outcome(natural_roll, total, intent.difficulty_class);

        // Generate a deterministic check_id from two RNG-produced u32 halves.
        let hi = u64::from(rng.next_u32_range(0, u32::MAX));
        let lo = u64::from(rng.next_u32_range(0, u32::MAX));
        let bits = (hi << 32) | lo;
        let check_id = Uuid::from_u64_pair(bits, bits.wrapping_mul(0x517c_c1b7_2722_0a95));

        let event = RulesEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "rules.check_resolved".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: RulesEventKind::CheckResolved(CheckResolved {
                resolution_id: self.id,
                check_id,
                natural_roll,
                modifier: intent.modifier,
                total,
                difficulty_class: intent.difficulty_class,
                outcome,
            }),
        };

        self.uncommitted_events.push(event);
        Ok(())
    }

    /// Produces effects from a resolved check, producing an `EffectsProduced` event.
    ///
    /// # Errors
    ///
    /// Returns `DomainError::Validation` if not in `CheckResolved` phase.
    pub fn produce_effects(
        &mut self,
        effects: Vec<ResolvedEffect>,
        correlation_id: Uuid,
        clock: &dyn Clock,
    ) -> Result<(), DomainError> {
        if self.phase != ResolutionPhase::CheckResolved {
            return Err(DomainError::Validation(
                "resolution must be in CheckResolved phase".to_owned(),
            ));
        }

        let event = RulesEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "rules.effects_produced".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: RulesEventKind::EffectsProduced(EffectsProduced {
                resolution_id: self.id,
                effects,
            }),
        };

        self.uncommitted_events.push(event);
        Ok(())
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
            RulesEventKind::IntentDeclared(payload) => {
                self.phase = ResolutionPhase::IntentDeclared;
                self.intent = Some(DeclaredIntent {
                    intent_id: payload.intent_id,
                    action_type: payload.action_type.clone(),
                    skill: payload.skill.clone(),
                    target_id: payload.target_id,
                    difficulty_class: payload.difficulty_class,
                    modifier: payload.modifier,
                });
            }
            RulesEventKind::CheckResolved(payload) => {
                self.phase = ResolutionPhase::CheckResolved;
                self.check_result = Some(CheckResult {
                    check_id: payload.check_id,
                    natural_roll: payload.natural_roll,
                    modifier: payload.modifier,
                    total: payload.total,
                    difficulty_class: payload.difficulty_class,
                    outcome: payload.outcome,
                });
            }
            RulesEventKind::EffectsProduced(payload) => {
                self.phase = ResolutionPhase::EffectsProduced;
                self.effects.clone_from(&payload.effects);
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
    use otherworlds_test_support::{FixedClock, SequenceRng};

    fn fixed_clock() -> FixedClock {
        FixedClock(Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap())
    }

    // --- declare_intent tests ---

    #[test]
    fn test_declare_intent_in_created_phase_produces_event() {
        let resolution_id = Uuid::new_v4();
        let intent_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let clock = fixed_clock();
        let mut resolution = Resolution::new(resolution_id);

        let result = resolution.declare_intent(
            intent_id,
            "skill_check".to_owned(),
            Some("perception".to_owned()),
            None,
            15,
            3,
            correlation_id,
            &clock,
        );

        assert!(result.is_ok());
        let events = resolution.uncommitted_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type(), "rules.intent_declared");

        let meta = events[0].metadata();
        assert_eq!(meta.aggregate_id, resolution_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
    }

    #[test]
    fn test_declare_intent_in_intent_declared_phase_returns_error() {
        let mut resolution = Resolution::new(Uuid::new_v4());
        resolution.phase = ResolutionPhase::IntentDeclared;

        let result = resolution.declare_intent(
            Uuid::new_v4(),
            "attack".to_owned(),
            None,
            None,
            12,
            0,
            Uuid::new_v4(),
            &fixed_clock(),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "resolution must be in Created phase");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_declare_intent_in_check_resolved_phase_returns_error() {
        let mut resolution = Resolution::new(Uuid::new_v4());
        resolution.phase = ResolutionPhase::CheckResolved;

        let result = resolution.declare_intent(
            Uuid::new_v4(),
            "save".to_owned(),
            None,
            None,
            10,
            2,
            Uuid::new_v4(),
            &fixed_clock(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_apply_intent_declared_updates_phase_and_intent() {
        let resolution_id = Uuid::new_v4();
        let intent_id = Uuid::new_v4();
        let mut resolution = Resolution::new(resolution_id);
        let event = RulesEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "rules.intent_declared".to_owned(),
                aggregate_id: resolution_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: Utc::now(),
            },
            kind: RulesEventKind::IntentDeclared(IntentDeclared {
                resolution_id,
                intent_id,
                action_type: "skill_check".to_owned(),
                skill: Some("athletics".to_owned()),
                target_id: None,
                difficulty_class: 15,
                modifier: 5,
            }),
        };

        resolution.apply(&event);

        assert_eq!(resolution.phase, ResolutionPhase::IntentDeclared);
        assert_eq!(resolution.version, 1);
        let intent = resolution.intent.as_ref().unwrap();
        assert_eq!(intent.intent_id, intent_id);
        assert_eq!(intent.action_type, "skill_check");
        assert_eq!(intent.skill.as_deref(), Some("athletics"));
        assert_eq!(intent.difficulty_class, 15);
        assert_eq!(intent.modifier, 5);
    }

    #[test]
    fn test_apply_intent_declared_preserves_fields() {
        let resolution_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let mut resolution = Resolution::new(resolution_id);
        let event = RulesEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "rules.intent_declared".to_owned(),
                aggregate_id: resolution_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: Utc::now(),
            },
            kind: RulesEventKind::IntentDeclared(IntentDeclared {
                resolution_id,
                intent_id: Uuid::new_v4(),
                action_type: "attack".to_owned(),
                skill: None,
                target_id: Some(target_id),
                difficulty_class: 12,
                modifier: -1,
            }),
        };

        resolution.apply(&event);

        let intent = resolution.intent.as_ref().unwrap();
        assert_eq!(intent.action_type, "attack");
        assert!(intent.skill.is_none());
        assert_eq!(intent.target_id, Some(target_id));
        assert_eq!(intent.modifier, -1);
    }

    #[test]
    fn test_sequence_number_increments() {
        let mut resolution = Resolution::new(Uuid::new_v4());
        let clock = fixed_clock();

        resolution
            .declare_intent(
                Uuid::new_v4(),
                "skill_check".to_owned(),
                None,
                None,
                15,
                0,
                Uuid::new_v4(),
                &clock,
            )
            .unwrap();

        assert_eq!(
            resolution.uncommitted_events()[0]
                .metadata()
                .sequence_number,
            1
        );
    }

    #[test]
    fn test_multiple_events_track_version() {
        let resolution_id = Uuid::new_v4();
        let mut resolution = Resolution::new(resolution_id);

        // Apply two events
        let event1 = RulesEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "rules.intent_declared".to_owned(),
                aggregate_id: resolution_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: Utc::now(),
            },
            kind: RulesEventKind::IntentDeclared(IntentDeclared {
                resolution_id,
                intent_id: Uuid::new_v4(),
                action_type: "skill_check".to_owned(),
                skill: None,
                target_id: None,
                difficulty_class: 15,
                modifier: 3,
            }),
        };
        let event2 = RulesEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "rules.check_resolved".to_owned(),
                aggregate_id: resolution_id,
                sequence_number: 2,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: Utc::now(),
            },
            kind: RulesEventKind::CheckResolved(CheckResolved {
                resolution_id,
                check_id: Uuid::new_v4(),
                natural_roll: 15,
                modifier: 3,
                total: 18,
                difficulty_class: 15,
                outcome: CheckOutcome::Success,
            }),
        };

        resolution.apply(&event1);
        resolution.apply(&event2);

        assert_eq!(resolution.version, 2);
        assert_eq!(resolution.phase, ResolutionPhase::CheckResolved);
    }

    // --- resolve_check tests ---

    #[test]
    fn test_resolve_check_in_intent_declared_phase_produces_event() {
        let resolution_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let clock = fixed_clock();
        let mut resolution = Resolution::new(resolution_id);

        // Set up the intent phase via apply
        resolution.phase = ResolutionPhase::IntentDeclared;
        resolution.intent = Some(DeclaredIntent {
            intent_id: Uuid::new_v4(),
            action_type: "skill_check".to_owned(),
            skill: None,
            target_id: None,
            difficulty_class: 15,
            modifier: 3,
        });

        // RNG: first value is the d20 roll (15), next two are for check_id generation
        let mut rng = SequenceRng::new(vec![15, 42, 99]);

        let result = resolution.resolve_check(correlation_id, &clock, &mut rng);
        assert!(result.is_ok());

        let events = resolution.uncommitted_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type(), "rules.check_resolved");

        match &events[0].kind {
            RulesEventKind::CheckResolved(payload) => {
                assert_eq!(payload.natural_roll, 15);
                assert_eq!(payload.modifier, 3);
                assert_eq!(payload.total, 18);
                assert_eq!(payload.difficulty_class, 15);
                assert_eq!(payload.outcome, CheckOutcome::Success);
            }
            other => panic!("expected CheckResolved, got {other:?}"),
        }
    }

    #[test]
    fn test_resolve_check_in_created_phase_returns_error() {
        let mut resolution = Resolution::new(Uuid::new_v4());
        let mut rng = SequenceRng::new(vec![10, 1, 1]);

        let result = resolution.resolve_check(Uuid::new_v4(), &fixed_clock(), &mut rng);

        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "resolution must be in IntentDeclared phase");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_resolve_check_in_check_resolved_phase_returns_error() {
        let mut resolution = Resolution::new(Uuid::new_v4());
        resolution.phase = ResolutionPhase::CheckResolved;
        let mut rng = SequenceRng::new(vec![10, 1, 1]);

        let result = resolution.resolve_check(Uuid::new_v4(), &fixed_clock(), &mut rng);

        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_check_natural_1_produces_critical_failure() {
        let mut resolution = Resolution::new(Uuid::new_v4());
        resolution.phase = ResolutionPhase::IntentDeclared;
        resolution.intent = Some(DeclaredIntent {
            intent_id: Uuid::new_v4(),
            action_type: "attack".to_owned(),
            skill: None,
            target_id: None,
            difficulty_class: 5,
            modifier: 10,
        });
        let mut rng = SequenceRng::new(vec![1, 42, 99]);

        resolution
            .resolve_check(Uuid::new_v4(), &fixed_clock(), &mut rng)
            .unwrap();

        match &resolution.uncommitted_events()[0].kind {
            RulesEventKind::CheckResolved(payload) => {
                assert_eq!(payload.natural_roll, 1);
                assert_eq!(payload.outcome, CheckOutcome::CriticalFailure);
            }
            other => panic!("expected CheckResolved, got {other:?}"),
        }
    }

    #[test]
    fn test_resolve_check_natural_20_produces_critical_success() {
        let mut resolution = Resolution::new(Uuid::new_v4());
        resolution.phase = ResolutionPhase::IntentDeclared;
        resolution.intent = Some(DeclaredIntent {
            intent_id: Uuid::new_v4(),
            action_type: "save".to_owned(),
            skill: None,
            target_id: None,
            difficulty_class: 30,
            modifier: -5,
        });
        let mut rng = SequenceRng::new(vec![20, 42, 99]);

        resolution
            .resolve_check(Uuid::new_v4(), &fixed_clock(), &mut rng)
            .unwrap();

        match &resolution.uncommitted_events()[0].kind {
            RulesEventKind::CheckResolved(payload) => {
                assert_eq!(payload.natural_roll, 20);
                assert_eq!(payload.outcome, CheckOutcome::CriticalSuccess);
            }
            other => panic!("expected CheckResolved, got {other:?}"),
        }
    }

    #[test]
    fn test_resolve_check_success_outcome() {
        let mut resolution = Resolution::new(Uuid::new_v4());
        resolution.phase = ResolutionPhase::IntentDeclared;
        resolution.intent = Some(DeclaredIntent {
            intent_id: Uuid::new_v4(),
            action_type: "skill_check".to_owned(),
            skill: Some("perception".to_owned()),
            target_id: None,
            difficulty_class: 15,
            modifier: 3,
        });
        // Roll 15 + modifier 3 = total 18, DC 15 → Success
        let mut rng = SequenceRng::new(vec![15, 42, 99]);

        resolution
            .resolve_check(Uuid::new_v4(), &fixed_clock(), &mut rng)
            .unwrap();

        match &resolution.uncommitted_events()[0].kind {
            RulesEventKind::CheckResolved(payload) => {
                assert_eq!(payload.total, 18);
                assert_eq!(payload.outcome, CheckOutcome::Success);
            }
            other => panic!("expected CheckResolved, got {other:?}"),
        }
    }

    #[test]
    fn test_resolve_check_partial_success_outcome() {
        let mut resolution = Resolution::new(Uuid::new_v4());
        resolution.phase = ResolutionPhase::IntentDeclared;
        resolution.intent = Some(DeclaredIntent {
            intent_id: Uuid::new_v4(),
            action_type: "skill_check".to_owned(),
            skill: None,
            target_id: None,
            difficulty_class: 15,
            modifier: 2,
        });
        // Roll 8 + modifier 2 = total 10, DC 15 → 10 >= 15-5 → PartialSuccess
        let mut rng = SequenceRng::new(vec![8, 42, 99]);

        resolution
            .resolve_check(Uuid::new_v4(), &fixed_clock(), &mut rng)
            .unwrap();

        match &resolution.uncommitted_events()[0].kind {
            RulesEventKind::CheckResolved(payload) => {
                assert_eq!(payload.total, 10);
                assert_eq!(payload.outcome, CheckOutcome::PartialSuccess);
            }
            other => panic!("expected CheckResolved, got {other:?}"),
        }
    }

    #[test]
    fn test_resolve_check_failure_outcome() {
        let mut resolution = Resolution::new(Uuid::new_v4());
        resolution.phase = ResolutionPhase::IntentDeclared;
        resolution.intent = Some(DeclaredIntent {
            intent_id: Uuid::new_v4(),
            action_type: "skill_check".to_owned(),
            skill: None,
            target_id: None,
            difficulty_class: 15,
            modifier: 1,
        });
        // Roll 3 + modifier 1 = total 4, DC 15 → 4 < 10 → Failure
        let mut rng = SequenceRng::new(vec![3, 42, 99]);

        resolution
            .resolve_check(Uuid::new_v4(), &fixed_clock(), &mut rng)
            .unwrap();

        match &resolution.uncommitted_events()[0].kind {
            RulesEventKind::CheckResolved(payload) => {
                assert_eq!(payload.total, 4);
                assert_eq!(payload.outcome, CheckOutcome::Failure);
            }
            other => panic!("expected CheckResolved, got {other:?}"),
        }
    }

    #[test]
    fn test_resolve_check_critical_success_by_margin() {
        let mut resolution = Resolution::new(Uuid::new_v4());
        resolution.phase = ResolutionPhase::IntentDeclared;
        resolution.intent = Some(DeclaredIntent {
            intent_id: Uuid::new_v4(),
            action_type: "skill_check".to_owned(),
            skill: None,
            target_id: None,
            difficulty_class: 5,
            modifier: 5,
        });
        // Roll 10 + modifier 5 = total 15, DC 5 → 15 >= 5+10 → CriticalSuccess
        let mut rng = SequenceRng::new(vec![10, 42, 99]);

        resolution
            .resolve_check(Uuid::new_v4(), &fixed_clock(), &mut rng)
            .unwrap();

        match &resolution.uncommitted_events()[0].kind {
            RulesEventKind::CheckResolved(payload) => {
                assert_eq!(payload.total, 15);
                assert_eq!(payload.outcome, CheckOutcome::CriticalSuccess);
            }
            other => panic!("expected CheckResolved, got {other:?}"),
        }
    }

    #[test]
    fn test_apply_check_resolved_updates_phase_and_result() {
        let resolution_id = Uuid::new_v4();
        let check_id = Uuid::new_v4();
        let mut resolution = Resolution::new(resolution_id);
        resolution.phase = ResolutionPhase::IntentDeclared;

        let event = RulesEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "rules.check_resolved".to_owned(),
                aggregate_id: resolution_id,
                sequence_number: 2,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: Utc::now(),
            },
            kind: RulesEventKind::CheckResolved(CheckResolved {
                resolution_id,
                check_id,
                natural_roll: 18,
                modifier: 3,
                total: 21,
                difficulty_class: 15,
                outcome: CheckOutcome::Success,
            }),
        };

        resolution.apply(&event);

        assert_eq!(resolution.phase, ResolutionPhase::CheckResolved);
        let result = resolution.check_result.as_ref().unwrap();
        assert_eq!(result.check_id, check_id);
        assert_eq!(result.natural_roll, 18);
        assert_eq!(result.total, 21);
        assert_eq!(result.outcome, CheckOutcome::Success);
    }

    // --- produce_effects tests ---

    #[test]
    fn test_produce_effects_in_check_resolved_phase_produces_event() {
        let resolution_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let clock = fixed_clock();
        let mut resolution = Resolution::new(resolution_id);
        resolution.phase = ResolutionPhase::CheckResolved;

        let effects = vec![ResolvedEffect {
            effect_type: "damage".to_owned(),
            target_id: Some(Uuid::new_v4()),
            payload: serde_json::json!({ "amount": 8 }),
        }];

        let result = resolution.produce_effects(effects, correlation_id, &clock);
        assert!(result.is_ok());

        let events = resolution.uncommitted_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type(), "rules.effects_produced");
    }

    #[test]
    fn test_produce_effects_in_created_phase_returns_error() {
        let mut resolution = Resolution::new(Uuid::new_v4());

        let result = resolution.produce_effects(vec![], Uuid::new_v4(), &fixed_clock());

        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "resolution must be in CheckResolved phase");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_produce_effects_in_intent_declared_phase_returns_error() {
        let mut resolution = Resolution::new(Uuid::new_v4());
        resolution.phase = ResolutionPhase::IntentDeclared;

        let result = resolution.produce_effects(vec![], Uuid::new_v4(), &fixed_clock());
        assert!(result.is_err());
    }

    #[test]
    fn test_produce_effects_in_effects_produced_phase_returns_error() {
        let mut resolution = Resolution::new(Uuid::new_v4());
        resolution.phase = ResolutionPhase::EffectsProduced;

        let result = resolution.produce_effects(vec![], Uuid::new_v4(), &fixed_clock());
        assert!(result.is_err());
    }

    #[test]
    fn test_produce_effects_event_contains_correct_effects() {
        let mut resolution = Resolution::new(Uuid::new_v4());
        resolution.phase = ResolutionPhase::CheckResolved;
        let target_id = Uuid::new_v4();

        let effects = vec![
            ResolvedEffect {
                effect_type: "damage".to_owned(),
                target_id: Some(target_id),
                payload: serde_json::json!({ "amount": 8 }),
            },
            ResolvedEffect {
                effect_type: "status_apply".to_owned(),
                target_id: Some(target_id),
                payload: serde_json::json!({ "status": "prone" }),
            },
        ];

        resolution
            .produce_effects(effects, Uuid::new_v4(), &fixed_clock())
            .unwrap();

        match &resolution.uncommitted_events()[0].kind {
            RulesEventKind::EffectsProduced(payload) => {
                assert_eq!(payload.effects.len(), 2);
                assert_eq!(payload.effects[0].effect_type, "damage");
                assert_eq!(payload.effects[1].effect_type, "status_apply");
            }
            other => panic!("expected EffectsProduced, got {other:?}"),
        }
    }

    #[test]
    fn test_apply_effects_produced_updates_phase_and_stores_effects() {
        let resolution_id = Uuid::new_v4();
        let mut resolution = Resolution::new(resolution_id);
        resolution.phase = ResolutionPhase::CheckResolved;

        let event = RulesEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "rules.effects_produced".to_owned(),
                aggregate_id: resolution_id,
                sequence_number: 3,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: Utc::now(),
            },
            kind: RulesEventKind::EffectsProduced(EffectsProduced {
                resolution_id,
                effects: vec![ResolvedEffect {
                    effect_type: "heal".to_owned(),
                    target_id: None,
                    payload: serde_json::json!({ "amount": 5 }),
                }],
            }),
        };

        resolution.apply(&event);

        assert_eq!(resolution.phase, ResolutionPhase::EffectsProduced);
        assert_eq!(resolution.effects.len(), 1);
        assert_eq!(resolution.effects[0].effect_type, "heal");
    }

    // --- Full lifecycle integration test ---

    #[test]
    fn test_full_lifecycle_declare_resolve_produce() {
        let resolution_id = Uuid::new_v4();
        let clock = fixed_clock();
        let mut resolution = Resolution::new(resolution_id);

        // Phase 1: Declare intent
        resolution
            .declare_intent(
                Uuid::new_v4(),
                "skill_check".to_owned(),
                Some("athletics".to_owned()),
                None,
                15,
                3,
                Uuid::new_v4(),
                &clock,
            )
            .unwrap();

        // Apply and clear to simulate persistence
        for event in resolution.uncommitted_events().to_vec() {
            resolution.apply(&event);
        }
        resolution.clear_uncommitted_events();

        // Phase 2: Resolve check
        let mut rng = SequenceRng::new(vec![15, 42, 99]);
        resolution
            .resolve_check(Uuid::new_v4(), &clock, &mut rng)
            .unwrap();

        for event in resolution.uncommitted_events().to_vec() {
            resolution.apply(&event);
        }
        resolution.clear_uncommitted_events();

        // Phase 3: Produce effects
        let effects = vec![ResolvedEffect {
            effect_type: "damage".to_owned(),
            target_id: None,
            payload: serde_json::json!({ "amount": 8 }),
        }];
        resolution
            .produce_effects(effects, Uuid::new_v4(), &clock)
            .unwrap();

        for event in resolution.uncommitted_events().to_vec() {
            resolution.apply(&event);
        }
        resolution.clear_uncommitted_events();

        // Final state
        assert_eq!(resolution.phase, ResolutionPhase::EffectsProduced);
        assert_eq!(resolution.version, 3);
        assert!(resolution.intent.is_some());
        assert!(resolution.check_result.is_some());
        assert_eq!(resolution.effects.len(), 1);
    }
}
