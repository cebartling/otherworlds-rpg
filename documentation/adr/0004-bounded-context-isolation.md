# ADR-0004: Bounded Context Isolation via Strict Dependency Direction

## Status

Accepted

## Context

The Otherworlds backend contains seven domain bounded contexts (Narrative, Rules, World State, Character, Inventory, Session, Content), a shared core library, an event store, and an API composition root. In a modular monolith, the risk of accidental coupling between domain modules is high — a single cross-domain import can create circular dependencies, leak implementation details, and make independent testing impossible.

Domain-Driven Design prescribes that bounded contexts should be autonomous. Each context should own its domain model, events, commands, and aggregates without depending on another context's internals.

## Decision

We enforce a strict dependency rule at the Cargo crate level:

**No domain crate may depend on another domain crate.**

### Dependency direction

```
otherworlds-api (binary, composition root)
    ├── otherworlds-narrative
    ├── otherworlds-rules
    ├── otherworlds-world-state
    ├── otherworlds-character
    ├── otherworlds-inventory
    ├── otherworlds-session
    ├── otherworlds-content
    ├── otherworlds-event-store
    └── otherworlds-core

All domain crates → otherworlds-core (only)
```

- Every domain crate lists only `otherworlds-core` (and external library dependencies) in its `[dependencies]`.
- `otherworlds-api` is the sole crate that depends on all others. It serves as the composition root, wiring up routes, state, and cross-context coordination.
- `otherworlds-event-store` depends on `otherworlds-core` for trait definitions and provides the persistence implementation.

### Cross-context communication

When one bounded context needs information or side effects from another:

1. **API-level orchestration**: The `otherworlds-api` binary coordinates across contexts by calling into multiple crate APIs within a single request handler.
2. **Event-driven reactions**: A context publishes domain events; the API layer or a future process manager subscribes and dispatches commands to other contexts.
3. **Shared abstractions**: Common trait definitions (`DomainEvent`, `Command`, `AggregateRoot`, etc.) live in `otherworlds-core`. Contexts implement these traits independently.

Direct crate-to-crate function calls between domain crates are prohibited.

### Internal crate structure

Each domain crate follows a consistent layout:

```
src/
├── lib.rs
├── domain/
│   ├── events.rs
│   ├── commands.rs
│   └── aggregates.rs
└── application/
    ├── command_handlers.rs
    └── query_handlers.rs
```

## Consequences

### Positive

- **Independent testing**: Each domain crate can be tested in isolation with `cargo test -p otherworlds-{context}`. No test fixtures from other domains are needed.
- **No circular dependencies**: The Cargo dependency graph is a strict DAG. The compiler enforces this — circular dependencies are a compile error.
- **Safe refactoring**: Changes to one context's internals cannot break another context, because no other context can see those internals.
- **Future extractability**: Since contexts have no compile-time coupling, extracting one into a standalone service requires only replacing in-process calls with network adapters at the API layer.

### Negative

- **Coordination overhead**: Cross-context workflows require explicit orchestration in `otherworlds-api` rather than simple function calls between modules. This is more verbose.
- **Potential duplication**: Contexts may define similar value objects independently rather than sharing them, because sharing would require pulling them into `otherworlds-core`.
- **Core bloat risk**: There is pressure to add shared types to `otherworlds-core`. This must be resisted — only true cross-cutting abstractions belong there.

### Constraints imposed

- Adding a domain crate dependency to another domain crate's `Cargo.toml` is a violation and must be caught in code review.
- `otherworlds-core` must contain only trait definitions and shared abstractions, not domain logic from any specific context.
- All cross-context coordination must be implemented in `otherworlds-api` or via event-driven process managers.
