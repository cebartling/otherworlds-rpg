# ADR-0009: Structured Domain Error Handling with thiserror

## Status

Accepted

## Context

The Otherworlds backend has multiple crates that need a consistent error handling strategy. Rust provides `Result<T, E>` and the `?` operator for ergonomic error propagation, but the ecosystem offers several approaches to defining error types: manual `impl Error`, `thiserror` for derive-based definitions, and `anyhow` for opaque error boxing.

The system needs:

- **Debuggable failures**: When a command fails, developers and operators must understand what went wrong and in what context. Opaque errors (e.g., "something went wrong") are insufficient.
- **Composable error propagation**: Domain crate errors must flow through command handlers, across the API layer, and into HTTP responses without losing context.
- **Consistent variants**: Multiple crates share failure modes (aggregate not found, concurrency conflict, validation error). A shared error enum prevents each crate from inventing its own error taxonomy.
- **No panics in production**: `.unwrap()` and `.expect()` in production code risk crashing the process and losing in-flight state.

## Decision

We define a single `DomainError` enum in `otherworlds-core` using `thiserror::Error` for derive-based error definitions. All domain crates propagate errors through this shared type.

### DomainError variants

```rust
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("aggregate not found: {0}")]
    AggregateNotFound(Uuid),

    #[error("concurrency conflict on aggregate {aggregate_id}: expected version {expected}, found {actual}")]
    ConcurrencyConflict {
        aggregate_id: Uuid,
        expected: i64,
        actual: i64,
    },

    #[error("validation error: {0}")]
    Validation(String),

    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
```

### Rules

- **`thiserror` for domain errors**: `DomainError` uses `thiserror::Error` derive to implement `std::error::Error` and `Display` with structured format strings.
- **No `.unwrap()` / `.expect()` outside tests**: Production code must use `?`, `map_err`, or explicit match arms to handle `Option` and `Result` values. Panicking in production is prohibited.
- **Propagation with `?`**: All functions that can fail return `Result<T, DomainError>` (or a type that converts to it). The `?` operator propagates errors up the call stack.
- **Context in messages**: Error variants include structured fields (UUIDs, version numbers) that provide enough context for debugging without requiring the caller to attach additional information.

### API layer mapping

The `otherworlds-api` crate defines its own `AppError` type that wraps `DomainError` and maps it to appropriate HTTP status codes:

- `AggregateNotFound` → 404
- `ConcurrencyConflict` → 409
- `Validation` → 400 or 422
- `Infrastructure` → 500

This separation keeps HTTP concerns out of domain crates.

## Consequences

### Positive

- **Debuggable failures**: Every error includes structured context (aggregate IDs, version numbers, descriptive messages). Operators can trace failures back to specific aggregates and operations.
- **Composable with `?`**: `thiserror` integrates with Rust's standard error traits, enabling clean error propagation across crate boundaries.
- **Consistent taxonomy**: All domain crates use the same error variants, so API layer mapping is uniform and predictable.
- **No panics**: Prohibiting `.unwrap()` / `.expect()` in production eliminates an entire class of process crashes.

### Negative

- **Shared error coupling**: All domain crates depend on `DomainError` from `otherworlds-core`. Adding a new error variant requires modifying the shared crate.
- **Variant growth**: As the system matures, `DomainError` may accumulate many variants. Periodic review may be needed to keep it focused.
- **Mapping overhead**: Each layer (domain → API → HTTP) must map errors, adding boilerplate to the API crate.

### Constraints imposed

- All domain error types must be variants of `DomainError` or convert into it.
- `.unwrap()` and `.expect()` are permitted only in `#[cfg(test)]` modules and test files.
- `anyhow` is not used for domain errors (it erases type information). It may be used in the API binary for top-level error handling if needed.
- New error variants must include sufficient context fields for debugging.
