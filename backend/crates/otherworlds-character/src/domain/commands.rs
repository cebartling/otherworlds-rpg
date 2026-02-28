//! Commands for the Character Management context.

use otherworlds_core::command::Command;
use uuid::Uuid;

/// Command to create a new character.
#[derive(Debug, Clone)]
pub struct CreateCharacter {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The character's name.
    pub name: String,
}

impl Command for CreateCharacter {
    fn command_type(&self) -> &'static str {
        "character.create_character"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
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

impl Command for ModifyAttribute {
    fn command_type(&self) -> &'static str {
        "character.modify_attribute"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
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

impl Command for AwardExperience {
    fn command_type(&self) -> &'static str {
        "character.award_experience"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}
