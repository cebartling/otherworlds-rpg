# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

### Rust Backend (from `backend/`)

```bash
cargo build                              # Dev build
cargo build --release                    # Release build
cargo run -p otherworlds-api             # Run API server
cargo test                               # All tests
cargo test -p otherworlds-narrative      # Single crate tests
cargo test test_name -- --nocapture      # Single test with output
cargo clippy --all-targets               # Lint (workspace lints: warn on clippy::all + clippy::pedantic, forbid unsafe)
cargo fmt -- --check                     # Format check
cargo check -p otherworlds-core         # Type-check single crate
```

### SvelteKit Web Client (from `web/`)

```bash
npm install                              # Install dependencies
npm run dev                              # Dev server
npm run build                            # Production build
npm run check                            # TypeScript/Svelte type checking
```

### Swift iOS Client (from `ios/OtherworldsApp/`)

```bash
swift build                              # Debug build
swift test                               # Run tests
```

### Docker (from root)

```bash
docker compose up postgres -d            # Database only
docker compose up                        # Full stack (postgres + api)
```

Environment: `DATABASE_URL`, `HOST` (default `0.0.0.0`), `PORT` (default `3000`), `RUST_LOG` (default `info`).

## Architecture

### Monorepo with Three Platforms

Rust backend (Axum modular monolith) serves both SvelteKit web and Swift/SwiftUI iOS clients over JSON HTTP. PostgreSQL 18 backs the event store. The backend is the single source of truth.

### Rust Workspace (edition 2024)

Ten crates under `backend/crates/`, one binary (`otherworlds-api`), nine libraries:

- **`otherworlds-core`** — Shared trait abstractions only: `DomainEvent`, `Command`, `AggregateRoot`, `EventRepository`, `Clock`, `DeterministicRng`, `DomainError`. Every other crate depends on this and nothing else.
- **`otherworlds-api`** — Composition root. The only crate that depends on all others. Builds the Axum router, creates `AppState` with `PgPool`, applies middleware (TraceLayer, CorsLayer), starts the server.
- **`otherworlds-event-store`** — PostgreSQL event persistence (currently `todo!()` stubs).
- **Six domain crates** — `narrative`, `rules`, `world-state`, `character`, `inventory`, `session`, `content`. Each represents a bounded context.

### Critical Dependency Rule

**No domain crate may depend on another domain crate.** All domain crates depend only on `otherworlds-core`. The `otherworlds-api` binary is the sole integration point. Violating this breaks bounded context isolation.

### Bounded Context Crate Structure

Each domain crate follows this internal layout:

```
src/
├── lib.rs                        # Re-exports pub modules
├── domain/
│   ├── events.rs                 # DomainEvent structs
│   ├── commands.rs               # Command structs
│   └── aggregates.rs             # AggregateRoot implementations
└── application/
    ├── command_handlers.rs        # Command → DomainEvent
    └── query_handlers.rs          # Read projections
```

### Event-Driven Flow

```
Command → CommandHandler → DomainEvent(s) → EventRepository.append_events() → State mutation via apply()
```

Commands never mutate state directly. All state changes are domain events. Events are append-only facts. Query handlers read projections, never mutate.

### Determinism Contract

The engine must be fully deterministic and replayable. Two injected traits enforce this:

- **`Clock`** — Abstracts time. Production uses `SystemClock`; tests inject fixed values.
- **`DeterministicRng`** — Abstracts randomness. Must be seedable and recordable.

AI may enhance narrative description text but **never determines state transitions**.

### API Route Pattern

Routes nest at `/api/v1/{context}` (e.g., `/api/v1/narrative`, `/api/v1/characters`). Health check at `/health` returns `{ status: "ok", version }`. Each context's router is defined in `src/routes/{context}.rs` and merged in `main.rs`.

### Event Metadata

Every domain event carries: `event_id` (UUID), `aggregate_id`, `sequence_number`, `correlation_id`, `causation_id`, `occurred_at`. Correlation IDs trace a command through its entire effect chain.

## Manifesto-Driven Development

Read `documentation/TECHNICAL_MANIFESTO.md` and `documentation/PRODUCT_MANIFESTO.md` before making architectural decisions. Architecture Decision Records live in `documentation/adr/` — consult existing ADRs before proposing changes and write a new ADR for any significant architectural decision. Six non-negotiable constraints: **Deterministic, Event-driven, Modular, Replayable, Versioned, Infrastructure-agnostic**. Any feature that introduces implicit state mutation, couples campaign logic to engine code, or breaks replay determinism must be rejected or redesigned.

## Test-Driven Development

All production code must be written using strict red-green TDD:

1. **Red** — Write a failing test first. Run it. Confirm it fails for the expected reason.
2. **Green** — Write the minimum production code to make the test pass. Nothing more.
3. **Refactor** — Clean up the implementation while keeping tests green. Run tests again.

Do not write production code without a corresponding failing test. Do not skip the red step. Do not write the test and production code simultaneously.

### TDD Workflow

- Write one test at a time. Do not batch multiple tests before implementing.
- Run the specific test after writing it to confirm the failure (`cargo test test_name` / `npm run check` / `swift test`).
- After making it pass, run the full crate/package test suite to check for regressions.
- Commit at green. Each commit should have passing tests.

### What to Test

- **Command handlers**: Given a command, assert the correct domain events are produced.
- **Aggregates**: Apply events to an aggregate, assert resulting state.
- **Domain logic**: Pure functions with deterministic inputs (injected `Clock`/`DeterministicRng`).
- **API endpoints**: Request in, expected response out (integration tests in `otherworlds-api`).
- **Projections/queries**: Given a sequence of events, assert the read model state.

### Test Structure

- Place unit tests in the same file as the code under test using `#[cfg(test)] mod tests`.
- Place integration tests in `tests/` directories within each crate.
- Name tests descriptively: `test_advance_beat_produces_beat_advanced_event`, not `test_1`.
- Use the Arrange-Act-Assert pattern.

## Code Conventions

- **Workspace lints**: `unsafe_code = "forbid"`, `clippy::all` and `clippy::pedantic` at warn level. All crates inherit via `[lints] workspace = true`.
- **Error types**: Use `thiserror::Error` with `DomainError` variants. Include context in messages. Propagate with `?`.
- **No `.unwrap()`/`.expect()`** outside tests.
- **File headers**: `//! {Context} — {responsibility}` with em-dash.
- **Crate naming**: `otherworlds-{context}` (kebab-case).
- **Serialization**: All domain types derive `Serialize`/`Deserialize`. UUIDs use v4/v7 with serde feature.
- **Async**: All I/O is async via Tokio. Trait async methods use `#[async_trait]`.
- **Indentation**: 4 spaces for Rust/Swift, 2 spaces for TS/Svelte/TOML/YAML/JSON (see `.editorconfig`).
- **Files end with newline**.

## Current State

The project is early-stage. Core abstractions, event store, and the narrative exemplar are implemented end-to-end.

- **`otherworlds-core`** — Trait abstractions fully defined (`EventRepository`, `Clock`, `DeterministicRng`, `DomainError`, etc.).
- **`otherworlds-event-store`** — `PgEventRepository` fully implemented with `load_events`, `append_events`, proactive optimistic concurrency control, batch INSERT via UNNEST, and tracing instrumentation. 10 integration tests.
- **`otherworlds-api`** — Server boots with `AppState` holding `PgPool`, `Clock`, `Rng`, and `PgEventRepository`. `ApiError` wrapper maps `DomainError` variants to HTTP status codes (404, 409, 400, 500). Narrative routes are live (`POST /advance-beat`, `POST /present-choice`). Other 6 context routers remain empty `Router::new()`. 10 unit tests (4 error mapping, 5 narrative handler, 1 state).
- **`otherworlds-narrative`** — Exemplar domain crate. `NarrativeSession` aggregate with `advance_beat()` and `present_choice()` domain methods. `handle_advance_beat` and `handle_present_choice` command handlers with full load-execute-persist flow. 4 unit tests.
- **Other domain crates** (`rules`, `world-state`, `character`, `inventory`, `session`, `content`) — Directory structure and type stubs exist. Command/query handler files are placeholders awaiting implementation.

Implementation should follow the established patterns in `otherworlds-narrative` (the exemplar).
