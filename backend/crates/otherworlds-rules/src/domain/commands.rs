//! Commands for the Rules & Resolution context.

use otherworlds_core::command::Command;
use uuid::Uuid;

/// Command to resolve a player intent.
#[derive(Debug, Clone)]
pub struct ResolveIntent {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The intent to resolve.
    pub intent_id: Uuid,
}

impl Command for ResolveIntent {
    fn command_type(&self) -> &'static str {
        "rules.resolve_intent"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}

/// Command to perform a check (skill, combat, etc.).
#[derive(Debug, Clone)]
pub struct PerformCheck {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The resolution this check belongs to.
    pub resolution_id: Uuid,
}

impl Command for PerformCheck {
    fn command_type(&self) -> &'static str {
        "rules.perform_check"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}
