//! Commands for the Character Management context.

use uuid::Uuid;

/// Command to create a new character.
#[derive(Debug, Clone)]
pub struct CreateCharacter {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The character's name.
    pub name: String,
}

/// Command to modify a character attribute.
#[derive(Debug, Clone)]
pub struct ModifyAttribute {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The character identifier.
    pub character_id: Uuid,
    /// The attribute key.
    pub attribute: String,
    /// The new value.
    pub new_value: i32,
}

/// Command to award experience to a character.
#[derive(Debug, Clone)]
pub struct AwardExperience {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The character identifier.
    pub character_id: Uuid,
    /// The amount of experience to award.
    pub amount: u32,
}
