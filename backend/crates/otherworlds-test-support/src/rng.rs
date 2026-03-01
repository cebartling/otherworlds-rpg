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

/// An RNG that returns values from predetermined sequences. Panics if either
/// sequence is exhausted. Used in tests that need specific, repeatable random
/// outcomes (e.g., dice rolls in the rules context).
///
/// Values returned by `next_u32_range` are clamped to `[min, max]` to ensure
/// the mock produces values within the same bounds as production code.
#[derive(Debug)]
pub struct SequenceRng {
    u32_values: Vec<u32>,
    u32_index: usize,
    f64_values: Vec<f64>,
    f64_index: usize,
}

impl SequenceRng {
    /// Create a new `SequenceRng` with the given u32 values. `next_f64` will
    /// return `0.0` for every call.
    #[must_use]
    pub fn new(values: Vec<u32>) -> Self {
        Self {
            u32_values: values,
            u32_index: 0,
            f64_values: Vec::new(),
            f64_index: 0,
        }
    }

    /// Create a new `SequenceRng` with both u32 and f64 value sequences.
    #[must_use]
    pub fn with_f64(u32_values: Vec<u32>, f64_values: Vec<f64>) -> Self {
        Self {
            u32_values,
            u32_index: 0,
            f64_values,
            f64_index: 0,
        }
    }
}

impl DeterministicRng for SequenceRng {
    fn next_u32_range(&mut self, min: u32, max: u32) -> u32 {
        let val = self.u32_values[self.u32_index];
        self.u32_index += 1;
        val.clamp(min, max)
    }

    fn next_f64(&mut self) -> f64 {
        if self.f64_values.is_empty() {
            return 0.0;
        }
        let val = self.f64_values[self.f64_index];
        self.f64_index += 1;
        val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_rng_returns_min() {
        let mut rng = MockRng;
        assert_eq!(rng.next_u32_range(5, 10), 5);
        assert_eq!(rng.next_u32_range(0, 100), 0);
    }

    #[test]
    fn test_mock_rng_returns_zero_f64() {
        let mut rng = MockRng;
        assert!((rng.next_f64() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sequence_rng_returns_values_in_order() {
        let mut rng = SequenceRng::new(vec![10, 20, 30]);
        assert_eq!(rng.next_u32_range(0, 100), 10);
        assert_eq!(rng.next_u32_range(0, 100), 20);
        assert_eq!(rng.next_u32_range(0, 100), 30);
    }

    #[test]
    fn test_sequence_rng_clamps_to_bounds() {
        let mut rng = SequenceRng::new(vec![50]);
        // Value 50 should be clamped to max = 10 when range is [0, 10].
        assert_eq!(rng.next_u32_range(0, 10), 10);
    }

    #[test]
    fn test_sequence_rng_clamps_below_min() {
        let mut rng = SequenceRng::new(vec![2]);
        // Value 2 should be clamped up to min = 5 when range is [5, 10).
        assert_eq!(rng.next_u32_range(5, 10), 5);
    }

    #[test]
    fn test_sequence_rng_f64_defaults_to_zero() {
        let mut rng = SequenceRng::new(vec![1]);
        assert!((rng.next_f64() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sequence_rng_with_f64_returns_sequenced_values() {
        let mut rng = SequenceRng::with_f64(vec![1], vec![0.5, 0.75]);
        assert!((rng.next_f64() - 0.5).abs() < f64::EPSILON);
        assert!((rng.next_f64() - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_sequence_rng_panics_when_u32_exhausted() {
        let mut rng = SequenceRng::new(vec![1]);
        rng.next_u32_range(0, 10);
        rng.next_u32_range(0, 10); // exhausted
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_sequence_rng_panics_when_f64_exhausted() {
        let mut rng = SequenceRng::with_f64(vec![], vec![0.5]);
        rng.next_f64();
        rng.next_f64(); // exhausted
    }
}
