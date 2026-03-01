//! Test clock — deterministic `Clock` implementation for tests.

use chrono::{DateTime, Utc};
use otherworlds_core::clock::Clock;

/// A clock that always returns a fixed point in time.
#[derive(Debug, Clone, Copy)]
pub struct FixedClock(pub DateTime<Utc>);

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.0
    }
}
