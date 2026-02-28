//! Otherworlds Event Store — PostgreSQL-backed event persistence.
//!
//! Schema is managed via sqlx migrations in `backend/migrations/`.

pub mod pg_event_repository;
