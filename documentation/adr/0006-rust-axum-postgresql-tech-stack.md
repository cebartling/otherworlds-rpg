# ADR-0006: Rust, Axum, and PostgreSQL Tech Stack

## Status

Accepted

## Context

Otherworlds RPG requires a backend that is:

- **Deterministic and safe** — The engine must not crash, corrupt state, or produce undefined behavior. Memory safety is non-negotiable for a system whose correctness depends on faithful event replay.
- **Performant** — Event sourcing involves replaying potentially long event streams to reconstruct state. The runtime must handle this efficiently.
- **Async-capable** — Multiple concurrent campaign sessions, database queries, and potential future WebSocket connections require an async I/O model.
- **Deployable as a single binary** — The modular monolith architecture ([ADR-0002](0002-modular-monolith-architecture.md)) calls for one deployable artifact with minimal runtime dependencies.

The system also needs a relational database for the event store with strong ACID guarantees, JSONB support for event payloads, and mature tooling.

Frontend clients include a SvelteKit web application and a Swift/SwiftUI iOS application, both consuming the same JSON HTTP API.

## Decision

### Backend

- **Language**: Rust (edition 2024) — Provides memory safety without garbage collection, zero-cost abstractions, and a strong type system that catches errors at compile time.
- **Web framework**: Axum 0.8 — Tower-based, async-first, integrates well with the Tokio ecosystem. Supports typed extractors, middleware composition, and state sharing via `State<AppState>`.
- **Async runtime**: Tokio 1.x — The de facto Rust async runtime. Provides the executor, timers, and I/O primitives.
- **Database driver**: sqlx 0.8 — Compile-time checked SQL queries, async PostgreSQL support, connection pooling, and migration management.
- **Serialization**: serde 1.0 + serde_json — Standard Rust serialization. All domain types derive `Serialize`/`Deserialize`.
- **Error handling**: thiserror 2.0 — Derives `std::error::Error` implementations for domain error types (see [ADR-0009](0009-structured-domain-error-handling.md)).
- **Observability**: tracing + tracing-subscriber — Structured, async-aware logging with JSON output and environment-based filtering.
- **IDs**: uuid 1.x (v4 and v7) — Universally unique identifiers for events, aggregates, and correlation chains.
- **Time**: chrono 0.4 — Date/time types with serde support, used behind the `Clock` trait abstraction.

### Database

- **PostgreSQL 18** — Relational database backing the event store, projections, and metadata. Chosen for ACID transactions, JSONB columns (event payloads), unique constraints (optimistic concurrency on `(aggregate_id, sequence_number)`), and mature ecosystem.

### Deployment

- **Docker** — Multi-stage builds produce minimal runtime images. `docker-compose.yml` defines the local development stack (PostgreSQL + API).
- **Target**: Digital Ocean — App Platform or managed Kubernetes, with managed PostgreSQL and object storage.

### Frontends

- **Web**: SvelteKit — Server-side rendering, TypeScript, typed API integration.
- **iOS**: Swift 6, SwiftUI — Native iOS experience, SPM for dependency management.

Both frontends are consumers of the backend JSON HTTP API. The backend is the single source of truth.

## Consequences

### Positive

- **Memory safety**: Rust's ownership model eliminates data races, use-after-free, and buffer overflows without runtime overhead.
- **Performance**: No garbage collector pauses. Event replay and aggregate reconstruction benefit from Rust's zero-cost abstractions.
- **Single binary deployment**: `cargo build --release` produces one statically-linked binary. No runtime, no JVM, no interpreter to install.
- **Compile-time guarantees**: Rust's type system and sqlx's compile-time query checking catch entire classes of errors before deployment.
- **Ecosystem coherence**: Axum, Tokio, sqlx, serde, and tracing are all part of the same Rust async ecosystem and integrate seamlessly.

### Negative

- **Steeper learning curve**: Rust's ownership and lifetime system has a significant learning curve compared to languages like TypeScript, Python, or Go.
- **Smaller talent pool**: Fewer developers have production Rust experience, which may affect team scaling.
- **Compile times**: Rust compilation (especially with sqlx compile-time checks) is slower than interpreted or JIT-compiled languages.
- **Library maturity**: While improving rapidly, some Rust libraries are less mature than equivalents in older ecosystems (Java, Python, Node.js).

### Constraints imposed

- All backend code must be Rust. No mixed-language backend (e.g., no Python microservices alongside Rust).
- The database must be PostgreSQL. Event store queries, concurrency controls, and migrations are PostgreSQL-specific.
- All async code must use Tokio as the runtime. Mixing runtimes (e.g., async-std) is prohibited.
