//! Random number generator abstraction for determinism.
//!
//! In production, this wraps a real RNG. In tests and replays,
//! a seeded or recorded implementation is injected.

use rand::Rng;

/// Abstraction over random number generation.
///
/// Methods take `&mut self` because RNG is inherently stateful. Concrete
/// implementations shared across threads must use interior mutability
/// (e.g., `Mutex<SeedableRng>`) to satisfy the `Send + Sync` bounds.
pub trait DeterministicRng: Send + Sync + std::fmt::Debug {
    /// Generate a random `u32` in the range `[min, max]` inclusive.
    fn next_u32_range(&mut self, min: u32, max: u32) -> u32;

    /// Generate a random `f64` in `[0.0, 1.0)`.
    fn next_f64(&mut self) -> f64;
}

/// Production RNG backed by the thread-local random number generator.
#[derive(Debug)]
pub struct StdRng;

impl DeterministicRng for StdRng {
    fn next_u32_range(&mut self, min: u32, max: u32) -> u32 {
        rand::rng().random_range(min..=max)
    }

    fn next_f64(&mut self) -> f64 {
        rand::rng().random::<f64>()
    }
}
