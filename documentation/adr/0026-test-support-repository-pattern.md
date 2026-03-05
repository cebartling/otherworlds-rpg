# ADR-0026: Test Support Repository Pattern

## Status

Accepted

## Context

The Otherworlds RPG engine uses event sourcing (ADR-0001) with an `EventRepository` trait that abstracts event persistence. Command handlers, aggregates, and orchestrators depend on this trait, not on PostgreSQL directly.

Testing these components against a real database would be slow, require infrastructure, and conflate domain logic correctness with database behavior. The test pyramid demands fast, isolated unit tests at the domain layer.

However, different test scenarios require different repository behaviors:

- **Happy path**: Load pre-configured events, record appended events for assertion.
- **Cross-context orchestration**: Load events for multiple distinct aggregates within a single test (e.g., branching clones events from narrative and character aggregates).
- **Error paths**: Simulate infrastructure failures, concurrency conflicts, or empty aggregate streams.

A single mock repository cannot serve all of these needs without becoming unwieldy.

## Decision

The `otherworlds-test-support` crate provides five specialized `EventRepository` implementations, each optimized for a specific test scenario:

### 1. `RecordingEventRepository`

The general-purpose happy-path repository. Configured with a fixed `Vec<StoredEvent>` that is returned from every `load_events` call regardless of aggregate ID. Records all `append_events` calls for later assertion. Suitable for single-aggregate command handler tests.

### 2. `MultiAggregateEventRepository`

A `HashMap<Uuid, Vec<StoredEvent>>` that returns different events for different aggregate IDs. Critically, `append_events` also stores the appended events in the map, making them available to subsequent `load_events` calls within the same test. This enables testing cross-context orchestration (ADR-0014, ADR-0021) where the orchestrator loads from one aggregate, creates another, and then loads the newly created aggregate.

### 3. `EmptyEventRepository`

Always returns an empty event list and silently accepts appends. Used for testing "aggregate not found" scenarios and creation commands where no prior state exists.

### 4. `ConflictingEventRepository`

Loads events successfully but always returns `DomainError::ConcurrencyConflict` on `append_events`. Configured with specific aggregate ID, expected version, and actual version for precise error assertions.

### 5. `FailingEventRepository`

Always returns `DomainError::Infrastructure("connection refused")` on every operation. Used for testing error propagation through command handlers and API routes.

### Shared Test Utilities

The crate also exports `FixedClock`, `MockRng`, and `SequenceRng` — deterministic implementations of the `Clock` and `DeterministicRng` traits (ADR-0003) for reproducible test execution.

## Consequences

### Easier

- Domain tests run in microseconds with no infrastructure dependencies.
- Each test scenario uses the simplest repository that fits — no over-mocking or complex setup.
- `MultiAggregateEventRepository` enables realistic integration-style tests for cross-context orchestration without a database.
- New domain crates automatically get test infrastructure by depending on `otherworlds-test-support`.

### More Difficult

- Five repository types must be understood by contributors writing tests — though their names and doc comments make the choice straightforward.
- `RecordingEventRepository` returns the same events for any aggregate ID, which can mask bugs where the wrong aggregate ID is passed. `MultiAggregateEventRepository` should be preferred when aggregate identity matters.
- Mock repositories do not enforce unique constraints or sequence number ordering — tests must construct valid event sequences manually.

### Unchanged

- Production code depends only on the `EventRepository` trait — it is unaware of test implementations.
- The `PgEventRepository` integration tests in `otherworlds-event-store` still test real database behavior.
- Bounded context isolation is preserved — test support is a dev dependency only.
