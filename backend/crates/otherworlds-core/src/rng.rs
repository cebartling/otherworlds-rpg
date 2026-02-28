//! Random number generator abstraction for determinism.
//!
//! In production, this wraps a real RNG. In tests and replays,
//! a seeded or recorded implementation is injected.

/// Abstraction over random number generation.
pub trait DeterministicRng: Send + Sync {
    /// Generate a random `u32` in the range `[min, max]` inclusive.
    fn next_u32_range(&mut self, min: u32, max: u32) -> u32;

    /// Generate a random `f64` in `[0.0, 1.0)`.
    fn next_f64(&mut self) -> f64;
}
