//! Otherworlds Core — shared domain abstractions.
//!
//! This crate defines the fundamental traits and types that all bounded
//! contexts depend on. It contains no infrastructure code.

pub mod aggregate;
pub mod clock;
pub mod command;
pub mod error;
pub mod event;
pub mod repository;
pub mod rng;
