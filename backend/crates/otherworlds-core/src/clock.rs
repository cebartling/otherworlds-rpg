//! Clock abstraction for determinism.

use chrono::{DateTime, Utc};

/// Abstraction over system time for deterministic behavior.
pub trait Clock: Send + Sync {
    /// Returns the current time.
    fn now(&self) -> DateTime<Utc>;
}

/// Production clock that delegates to the system clock.
#[derive(Debug, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
