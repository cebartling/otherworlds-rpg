//! Domain events for the Rules & Resolution context.

use std::fmt;

use otherworlds_core::event::{DomainEvent, EventMetadata};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Five-tier outcome of a d20 check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckOutcome {
    /// Natural 1 or catastrophic failure.
    CriticalFailure,
    /// Total below DC-5.
    Failure,
    /// Total in [DC-5, DC).
    PartialSuccess,
    /// Total meets or exceeds DC.
    Success,
    /// Natural 20 or total >= DC+10.
    CriticalSuccess,
}

impl fmt::Display for CheckOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CriticalFailure => write!(f, "critical_failure"),
            Self::Failure => write!(f, "failure"),
            Self::PartialSuccess => write!(f, "partial_success"),
            Self::Success => write!(f, "success"),
            Self::CriticalSuccess => write!(f, "critical_success"),
        }
    }
}

/// Determines the outcome of a d20 check.
///
/// - Natural 1: always `CriticalFailure`
/// - Natural 20: always `CriticalSuccess`
/// - total >= dc + 10: `CriticalSuccess`
/// - total >= dc: `Success`
/// - total >= dc - 5: `PartialSuccess`
/// - otherwise: `Failure`
#[must_use]
pub fn determine_outcome(natural_roll: u32, total: i32, dc: i32) -> CheckOutcome {
    if natural_roll == 1 {
        return CheckOutcome::CriticalFailure;
    }
    if natural_roll == 20 {
        return CheckOutcome::CriticalSuccess;
    }
    if total >= dc + 10 {
        CheckOutcome::CriticalSuccess
    } else if total >= dc {
        CheckOutcome::Success
    } else if total >= dc - 5 {
        CheckOutcome::PartialSuccess
    } else {
        CheckOutcome::Failure
    }
}

/// Emitted when a player declares an intent to act.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentDeclared {
    /// The resolution identifier.
    pub resolution_id: Uuid,
    /// The intent identifier.
    pub intent_id: Uuid,
    /// The type of action (e.g., "`skill_check`", "`attack`", "`save`").
    pub action_type: String,
    /// Optional skill being used (e.g., "perception", "athletics").
    pub skill: Option<String>,
    /// Optional target of the action.
    pub target_id: Option<Uuid>,
    /// The difficulty class to beat.
    pub difficulty_class: i32,
    /// The modifier applied to the roll.
    pub modifier: i32,
}

/// Emitted when a d20 check is resolved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResolved {
    /// The resolution identifier.
    pub resolution_id: Uuid,
    /// The check identifier.
    pub check_id: Uuid,
    /// The raw d20 result (1-20).
    pub natural_roll: u32,
    /// The modifier applied to the roll.
    pub modifier: i32,
    /// The total (`natural_roll` as i32 + modifier).
    pub total: i32,
    /// The difficulty class.
    pub difficulty_class: i32,
    /// The five-tier outcome.
    pub outcome: CheckOutcome,
}

/// A single campaign-independent effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedEffect {
    /// The type of effect (e.g., "`damage`", "`heal`", "`status_apply`").
    pub effect_type: String,
    /// Optional target of the effect.
    pub target_id: Option<Uuid>,
    /// Campaign-specific payload.
    pub payload: serde_json::Value,
}

/// Emitted when effects are produced from a resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectsProduced {
    /// The resolution identifier.
    pub resolution_id: Uuid,
    /// The produced effects.
    pub effects: Vec<ResolvedEffect>,
}

/// Event payload variants for the Rules & Resolution context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RulesEventKind {
    /// A player intent has been declared.
    IntentDeclared(IntentDeclared),
    /// A d20 check has been resolved.
    CheckResolved(CheckResolved),
    /// Effects have been produced from a resolution.
    EffectsProduced(EffectsProduced),
}

/// Domain event envelope for the Rules & Resolution context.
#[derive(Debug, Clone)]
pub struct RulesEvent {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// Event-specific payload.
    pub kind: RulesEventKind,
}

impl DomainEvent for RulesEvent {
    fn event_type(&self) -> &'static str {
        match &self.kind {
            RulesEventKind::IntentDeclared(_) => "rules.intent_declared",
            RulesEventKind::CheckResolved(_) => "rules.check_resolved",
            RulesEventKind::EffectsProduced(_) => "rules.effects_produced",
        }
    }

    fn to_payload(&self) -> serde_json::Value {
        serde_json::to_value(&self.kind).expect("RulesEventKind serialization is infallible")
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- determine_outcome tests ---

    #[test]
    fn test_natural_1_always_critical_failure() {
        assert_eq!(determine_outcome(1, 15, 10), CheckOutcome::CriticalFailure);
    }

    #[test]
    fn test_natural_1_overrides_high_total() {
        // Natural 1 is CriticalFailure even if total >= DC+10
        assert_eq!(determine_outcome(1, 25, 15), CheckOutcome::CriticalFailure);
    }

    #[test]
    fn test_natural_20_always_critical_success() {
        assert_eq!(determine_outcome(20, 5, 30), CheckOutcome::CriticalSuccess);
    }

    #[test]
    fn test_total_at_dc_plus_10_is_critical_success() {
        // total = 25, dc = 15 → 25 >= 15+10 → CriticalSuccess
        assert_eq!(determine_outcome(15, 25, 15), CheckOutcome::CriticalSuccess);
    }

    #[test]
    fn test_total_at_dc_plus_9_is_success() {
        // total = 24, dc = 15 → 24 < 25 → Success
        assert_eq!(determine_outcome(14, 24, 15), CheckOutcome::Success);
    }

    #[test]
    fn test_total_equals_dc_is_success() {
        // total = 15, dc = 15 → Success
        assert_eq!(determine_outcome(10, 15, 15), CheckOutcome::Success);
    }

    #[test]
    fn test_total_above_dc_is_success() {
        // total = 18, dc = 15 → Success
        assert_eq!(determine_outcome(12, 18, 15), CheckOutcome::Success);
    }

    #[test]
    fn test_total_at_dc_minus_1_is_partial_success() {
        // total = 14, dc = 15 → 14 >= 10 → PartialSuccess
        assert_eq!(determine_outcome(10, 14, 15), CheckOutcome::PartialSuccess);
    }

    #[test]
    fn test_total_at_dc_minus_5_is_partial_success() {
        // total = 10, dc = 15 → 10 >= 10 → PartialSuccess
        assert_eq!(determine_outcome(8, 10, 15), CheckOutcome::PartialSuccess);
    }

    #[test]
    fn test_total_at_dc_minus_6_is_failure() {
        // total = 9, dc = 15 → 9 < 10 → Failure
        assert_eq!(determine_outcome(7, 9, 15), CheckOutcome::Failure);
    }

    #[test]
    fn test_total_well_below_dc_is_failure() {
        // total = 4, dc = 15 → Failure
        assert_eq!(determine_outcome(3, 4, 15), CheckOutcome::Failure);
    }

    #[test]
    fn test_natural_20_overrides_low_total() {
        // Natural 20 with negative modifier, total below DC
        assert_eq!(determine_outcome(20, 10, 25), CheckOutcome::CriticalSuccess);
    }

    #[test]
    fn test_check_outcome_serialization_round_trip() {
        let outcome = CheckOutcome::PartialSuccess;
        let json = serde_json::to_string(&outcome).unwrap();
        let deserialized: CheckOutcome = serde_json::from_str(&json).unwrap();
        assert_eq!(outcome, deserialized);
    }

    // --- Event serialization round-trip tests ---

    #[test]
    fn test_intent_declared_serialization_round_trip() {
        let kind = RulesEventKind::IntentDeclared(IntentDeclared {
            resolution_id: Uuid::new_v4(),
            intent_id: Uuid::new_v4(),
            action_type: "skill_check".to_owned(),
            skill: Some("perception".to_owned()),
            target_id: None,
            difficulty_class: 15,
            modifier: 3,
        });
        let json = serde_json::to_value(&kind).unwrap();
        let deserialized: RulesEventKind = serde_json::from_value(json).unwrap();
        match deserialized {
            RulesEventKind::IntentDeclared(payload) => {
                assert_eq!(payload.action_type, "skill_check");
                assert_eq!(payload.difficulty_class, 15);
                assert_eq!(payload.modifier, 3);
            }
            other => panic!("expected IntentDeclared, got {other:?}"),
        }
    }

    #[test]
    fn test_check_resolved_serialization_round_trip() {
        let kind = RulesEventKind::CheckResolved(CheckResolved {
            resolution_id: Uuid::new_v4(),
            check_id: Uuid::new_v4(),
            natural_roll: 15,
            modifier: 3,
            total: 18,
            difficulty_class: 15,
            outcome: CheckOutcome::Success,
        });
        let json = serde_json::to_value(&kind).unwrap();
        let deserialized: RulesEventKind = serde_json::from_value(json).unwrap();
        match deserialized {
            RulesEventKind::CheckResolved(payload) => {
                assert_eq!(payload.natural_roll, 15);
                assert_eq!(payload.total, 18);
                assert_eq!(payload.outcome, CheckOutcome::Success);
            }
            other => panic!("expected CheckResolved, got {other:?}"),
        }
    }

    #[test]
    fn test_effects_produced_serialization_round_trip() {
        let kind = RulesEventKind::EffectsProduced(EffectsProduced {
            resolution_id: Uuid::new_v4(),
            effects: vec![ResolvedEffect {
                effect_type: "damage".to_owned(),
                target_id: Some(Uuid::new_v4()),
                payload: serde_json::json!({ "amount": 8 }),
            }],
        });
        let json = serde_json::to_value(&kind).unwrap();
        let deserialized: RulesEventKind = serde_json::from_value(json).unwrap();
        match deserialized {
            RulesEventKind::EffectsProduced(payload) => {
                assert_eq!(payload.effects.len(), 1);
                assert_eq!(payload.effects[0].effect_type, "damage");
            }
            other => panic!("expected EffectsProduced, got {other:?}"),
        }
    }

    #[test]
    fn test_resolved_effect_serialization_round_trip() {
        let effect = ResolvedEffect {
            effect_type: "status_apply".to_owned(),
            target_id: None,
            payload: serde_json::json!({ "status": "poisoned", "duration": 3 }),
        };
        let json = serde_json::to_value(&effect).unwrap();
        let deserialized: ResolvedEffect = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.effect_type, "status_apply");
        assert!(deserialized.target_id.is_none());
    }
}
