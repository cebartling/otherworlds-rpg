# ADR-0001: Event Sourcing and CQRS

## Status

Accepted

## Context

Otherworlds RPG is a deterministic narrative engine where every playthrough produces an ordered, replayable history of events. The system requires:

- **Deterministic replay** — Given the same inputs and seed, a campaign run must produce identical results. This rules out any persistence model that loses intermediate state.
- **Auditable outcomes** — Players and authors must be able to inspect why something happened. Every state transition needs a traceable cause.
- **Branching timelines** — The Session & Progress bounded context supports save/restore and branching, which requires reconstructing state at any point in the event stream.
- **Transparent state transitions** — The product manifesto explicitly rejects hidden mechanics and implicit state mutation. All changes must be explicit, recorded facts.

Traditional CRUD persistence overwrites previous state, making replay impossible and audit trails an afterthought. An event log is a natural fit for a system whose core metaphor is "the world remembers."

## Decision

We adopt **Event Sourcing** as the persistence strategy and **CQRS** (Command Query Responsibility Segregation) for read/write separation.

### Event Sourcing

- All state changes are expressed as immutable, append-only **domain events**. Events are facts that have occurred and cannot be modified or deleted.
- Aggregates reconstitute their current state by replaying their event history through `apply()`. There is no separate "current state" table.
- Every event carries metadata for full traceability: `event_id` (UUID), `aggregate_id`, `sequence_number`, `correlation_id`, `causation_id`, and `occurred_at`.
- The event store is backed by **PostgreSQL**, with optimistic concurrency enforced via a unique constraint on `(aggregate_id, sequence_number)`.

### CQRS

- **Commands** represent intent (e.g., "advance the beat", "resolve a skill check"). Commands are handled by command handlers that validate business rules and produce zero or more domain events. Commands never mutate state directly.
- **Query handlers** read from projections (read models) built from events. Query handlers never mutate state.
- Command and query paths are separated at the API level: command endpoints accept commands and return the resulting events; query endpoints return projected state.

### Determinism enforcement

- Time is abstracted behind the `Clock` trait. Production uses `SystemClock`; tests and replays inject fixed or recorded values.
- Randomness is abstracted behind the `DeterministicRng` trait. Implementations must be seedable and recordable so that replays produce identical outcomes.
- AI may enhance narrative description text but never determines state transitions.

### Implementation in the codebase

The core abstractions live in `otherworlds-core` and are implemented across all bounded contexts:

- `DomainEvent` trait — requires `event_type()`, `to_payload()`, and `metadata()`.
- `Command` trait — requires `command_type()` and `correlation_id()`.
- `AggregateRoot` trait — requires `apply()`, `uncommitted_events()`, `clear_uncommitted_events()`, `aggregate_id()`, and `version()`.
- `EventRepository` trait — provides `load_events()` and `append_events()` with an `expected_version` parameter for optimistic concurrency.

The flow through the system is:

```
Command → CommandHandler → DomainEvent(s) → EventRepository.append_events() → State mutation via apply()
```

## Consequences

### Positive

- **Full replay**: Any campaign run can be reconstructed by replaying its event stream, enabling save/restore and time-travel debugging.
- **Branching timelines**: Forking a run at any point is straightforward — copy the event stream up to the branch point and diverge.
- **Audit trail**: Every state change is a recorded fact with correlation and causation IDs, making it possible to trace any outcome back to its original command.
- **Testability**: Command handlers are pure functions (command + state in, events out) that are trivial to unit test. Determinism traits eliminate flaky tests.
- **Bounded context isolation**: Each domain crate defines its own events and aggregates against `otherworlds-core` traits, with no cross-crate domain dependencies.

### Negative

- **Eventual consistency**: Read models (projections) may lag behind the event stream. Consumers must tolerate short windows of stale data.
- **Increased storage**: Storing every event rather than just current state requires more disk space, especially for long campaign runs.
- **Complexity**: Developers must understand the event sourcing and CQRS patterns. Debugging requires thinking in terms of event streams rather than current state.
- **Schema evolution**: Changing event structures requires versioned event upcasters to maintain compatibility with historical data.

### Constraints imposed

- No domain crate may perform direct state mutation. All state changes must flow through the Command → Event → Apply pattern.
- No CRUD-style updates. There is no "update character" — there is "apply CharacterAttributeChanged event."
- All domain types must derive `Serialize` and `Deserialize` for event persistence.
- All bounded contexts must depend only on `otherworlds-core` for shared abstractions. The `otherworlds-api` binary is the sole integration point.
