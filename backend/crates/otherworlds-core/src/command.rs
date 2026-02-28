//! Command abstractions.

use uuid::Uuid;

/// Trait that all commands implement.
pub trait Command: Send + Sync + std::fmt::Debug {
    /// The type name for this command (for logging/routing).
    fn command_type(&self) -> &'static str;

    /// Correlation ID to trace this command through the system.
    fn correlation_id(&self) -> Uuid;
}
