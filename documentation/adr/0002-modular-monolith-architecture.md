# ADR-0002: Modular Monolith Architecture

## Status

Accepted

## Context

Otherworlds RPG is a multi-bounded-context system with seven domain crates, a shared core, an event store, and an API composition root. Distributed architectures (microservices, serverless) offer independent scaling and deployment, but they introduce significant operational complexity: service discovery, distributed transactions, network partitioning, and deployment orchestration.

At the current stage the system has a single development team, a single deployment target (Digital Ocean), and no evidence that any bounded context requires independent scaling. The primary concerns are:

- **Iteration speed** — Rapid, cross-context changes without multi-service coordination.
- **Deployment simplicity** — One artifact to build, test, and deploy.
- **Strong invariants** — In-process communication allows compile-time guarantees across contexts.
- **Future extraction** — The architecture must not prevent breaking out contexts into services if scale eventually demands it.

## Decision

We structure the backend as a **modular monolith** using a Cargo workspace with one binary crate (`otherworlds-api`) and multiple library crates (one per bounded context, plus `otherworlds-core` and `otherworlds-event-store`).

### Structure

- The Cargo workspace lists all crates under `backend/crates/`.
- `otherworlds-api` is the sole binary and composition root. It depends on all other crates and assembles the Axum router, application state, and middleware.
- Domain crates communicate via in-process trait calls and shared abstractions from `otherworlds-core`. There is no inter-service messaging, no HTTP between contexts, and no shared mutable state beyond what `AppState` provides.
- Each crate is independently compilable and testable (`cargo test -p otherworlds-narrative`).

### Boundaries

- Crate boundaries enforce bounded context isolation at compile time.
- Dependency direction is strictly enforced: domain crates depend only on `otherworlds-core` (see [ADR-0004](0004-bounded-context-isolation.md)).
- Internal APIs between contexts are Rust trait objects and function calls, not network protocols.

### Future extraction

If a bounded context must be extracted into a standalone service:

1. Replace in-process trait calls with a messaging adapter (e.g., async channel, message queue, or HTTP client).
2. Externalize the context's event stream to a shared event bus.
3. Deploy the extracted crate as its own binary.

This path is available but not pre-built. Premature distribution is prohibited.

## Consequences

### Positive

- **Faster iteration**: Cross-context refactors are single-commit, single-build changes with immediate compiler feedback.
- **Simpler deployment**: One Docker image, one process, one health check. No service mesh, no container orchestration beyond a single container.
- **Stronger invariants**: Rust's type system enforces API contracts between contexts at compile time rather than at runtime.
- **Lower operational cost**: No distributed tracing required for intra-process calls. Fewer failure modes.

### Negative

- **Scaling is uniform**: All contexts scale together. If one context has disproportionate load, it cannot be scaled independently without extraction.
- **Single failure domain**: A panic or resource exhaustion in one context affects the entire process.
- **Extraction cost**: While the architecture permits future extraction, replacing in-process calls with network calls is non-trivial and requires adapter implementations.

### Constraints imposed

- All domain crates must remain library crates. Only `otherworlds-api` produces a binary.
- Cross-context communication must go through trait abstractions, not direct struct access, to preserve future extractability.
- The workspace must remain buildable and testable as a single `cargo build` / `cargo test` invocation.
