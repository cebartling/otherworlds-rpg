//! Domain events for the Rules & Resolution context.

use otherworlds_core::event::{DomainEvent, EventMetadata};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Emitted when a player intent has been resolved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResolved {
    /// The resolution identifier.
    pub resolution_id: Uuid,
    /// The intent that was resolved.
    pub intent_id: Uuid,
}

/// Emitted when a skill/combat check is performed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckPerformed {
    /// The resolution identifier.
    pub resolution_id: Uuid,
    /// The check identifier.
    pub check_id: Uuid,
}

/// Emitted when effects are produced from a resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectsProduced {
    /// The resolution identifier.
    pub resolution_id: Uuid,
}

/// Event payload variants for the Rules & Resolution context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RulesEventKind {
    /// A player intent has been resolved.
    IntentResolved(IntentResolved),
    /// A skill/combat check has been performed.
    CheckPerformed(CheckPerformed),
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
            RulesEventKind::IntentResolved(_) => "rules.intent_resolved",
            RulesEventKind::CheckPerformed(_) => "rules.check_performed",
            RulesEventKind::EffectsProduced(_) => "rules.effects_produced",
        }
    }

    fn to_payload(&self) -> serde_json::Value {
        // Serialization of derived Serialize types to Value is infallible.
        serde_json::to_value(&self.kind).expect("RulesEventKind serialization is infallible")
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
