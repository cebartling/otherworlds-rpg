//! Domain events for the Character Management context.

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
