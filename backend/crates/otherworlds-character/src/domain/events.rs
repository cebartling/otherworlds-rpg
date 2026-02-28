//! Domain events for the Character Management context.

use otherworlds_core::event::{DomainEvent, EventMetadata};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Emitted when a character is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterCreated {
    /// The character identifier.
    pub character_id: Uuid,
    /// The character's name.
    pub name: String,
}

/// Emitted when a character attribute is modified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeModified {
    /// The character identifier.
    pub character_id: Uuid,
    /// The attribute key.
    pub attribute: String,
    /// The new value.
    pub new_value: i32,
}

/// Emitted when a character gains experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceGained {
    /// The character identifier.
    pub character_id: Uuid,
    /// The amount of experience gained.
    pub amount: u32,
}

/// Event payload variants for the Character Management context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CharacterEventKind {
    /// A character has been created.
    CharacterCreated(CharacterCreated),
    /// A character attribute has been modified.
    AttributeModified(AttributeModified),
    /// A character has gained experience.
    ExperienceGained(ExperienceGained),
}

/// Domain event envelope for the Character Management context.
#[derive(Debug, Clone)]
pub struct CharacterEvent {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// Event-specific payload.
    pub kind: CharacterEventKind,
}

impl DomainEvent for CharacterEvent {
    fn event_type(&self) -> &'static str {
        match &self.kind {
            CharacterEventKind::CharacterCreated(_) => "character.character_created",
            CharacterEventKind::AttributeModified(_) => "character.attribute_modified",
            CharacterEventKind::ExperienceGained(_) => "character.experience_gained",
        }
    }

    fn to_payload(&self) -> serde_json::Value {
        // Serialization of derived Serialize types to Value is infallible.
        serde_json::to_value(&self.kind).expect("CharacterEventKind serialization is infallible")
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
