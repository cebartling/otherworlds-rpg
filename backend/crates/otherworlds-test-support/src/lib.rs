//! Shared test mocks and utilities for the Otherworlds RPG engine.

mod clock;
mod repository;
mod rng;

pub use clock::FixedClock;
pub use repository::{EmptyEventRepository, FailingEventRepository, RecordingEventRepository};
pub use rng::{MockRng, SequenceRng};
