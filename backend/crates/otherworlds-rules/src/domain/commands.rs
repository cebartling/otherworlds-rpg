//! Commands for the Rules & Resolution context.

use otherworlds_core::command::Command;
use uuid::Uuid;

use super::events::ResolvedEffect;

/// Command to declare a player intent.
#[derive(Debug, Clone)]
pub struct DeclareIntent {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The resolution this intent belongs to.
    pub resolution_id: Uuid,
    /// The intent identifier.
    pub intent_id: Uuid,
    /// The type of action (e.g., "`skill_check`", "`attack`", "`save`").
    pub action_type: String,
    /// Optional skill being used.
    pub skill: Option<String>,
    /// Optional target of the action.
    pub target_id: Option<Uuid>,
    /// The difficulty class to beat.
    pub difficulty_class: i32,
    /// The modifier applied to the roll.
    pub modifier: i32,
}

impl Command for DeclareIntent {
    fn command_type(&self) -> &'static str {
        "rules.declare_intent"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}

/// Command to resolve a d20 check.
#[derive(Debug, Clone)]
pub struct ResolveCheck {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The resolution this check belongs to.
    pub resolution_id: Uuid,
}

impl Command for ResolveCheck {
    fn command_type(&self) -> &'static str {
        "rules.resolve_check"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}

/// Specification for a single effect to produce.
#[derive(Debug, Clone)]
pub struct EffectSpec {
    /// The type of effect (e.g., "`damage`", "`heal`", "`status_apply`").
    pub effect_type: String,
    /// Optional target of the effect.
    pub target_id: Option<Uuid>,
    /// Campaign-specific payload.
    pub payload: serde_json::Value,
}

impl EffectSpec {
    /// Converts this spec into a `ResolvedEffect` for the domain.
    #[must_use]
    pub fn into_resolved_effect(self) -> ResolvedEffect {
        ResolvedEffect {
            effect_type: self.effect_type,
            target_id: self.target_id,
            payload: self.payload,
        }
    }
}

/// Command to produce effects from a resolved check.
#[derive(Debug, Clone)]
pub struct ProduceEffects {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The resolution this effect production belongs to.
    pub resolution_id: Uuid,
    /// The effects to produce.
    pub effects: Vec<EffectSpec>,
}

impl Command for ProduceEffects {
    fn command_type(&self) -> &'static str {
        "rules.produce_effects"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}
