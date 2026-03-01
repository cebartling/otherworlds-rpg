//! Test RNG — deterministic `DeterministicRng` implementations for tests.

use otherworlds_core::rng::DeterministicRng;

/// A no-op RNG that always returns `min` for `next_u32_range` and `0.0` for
/// `next_f64`. Suitable for tests that do not depend on specific random values.
#[derive(Debug)]
pub struct MockRng;

impl DeterministicRng for MockRng {
    fn next_u32_range(&mut self, min: u32, _max: u32) -> u32 {
        min
    }

    fn next_f64(&mut self) -> f64 {
        0.0
    }
}

/// An RNG that returns values from a predetermined sequence. Panics if the
/// sequence is exhausted. Used in tests that need specific, repeatable random
/// outcomes (e.g., dice rolls in the rules context).
#[derive(Debug)]
pub struct SequenceRng {
    values: Vec<u32>,
    index: usize,
}

impl SequenceRng {
    /// Create a new `SequenceRng` with the given values.
    #[must_use]
    pub fn new(values: Vec<u32>) -> Self {
        Self { values, index: 0 }
    }
}

impl DeterministicRng for SequenceRng {
    fn next_u32_range(&mut self, _min: u32, _max: u32) -> u32 {
        let val = self.values[self.index];
        self.index += 1;
        val
    }

    fn next_f64(&mut self) -> f64 {
        0.0
    }
}
