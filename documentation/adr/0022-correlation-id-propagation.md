# ADR-0022: Correlation ID Propagation Across All Platforms

## Status

Accepted

## Context

The Otherworlds RPG engine is event-sourced (ADR-0001) and orchestrates multiple bounded contexts per request (ADR-0014). When a single user action produces events across rules, world state, and narrative contexts, there must be a way to trace all of those events back to the originating request.

Without a consistent correlation strategy:

1. **Debugging is impossible** — an event in the world-state context cannot be linked to the rules check that caused it.
2. **Replay auditing breaks** — the manifesto requires full replayability, which means understanding causal chains.
3. **Observability gaps** — distributed traces across backend, web, and iOS cannot be correlated.

## Decision

Every domain event and command carries a `correlation_id` (UUID) as mandatory metadata. The ID is propagated through all layers:

### Core Contract

- The `Command` trait requires a `correlation_id()` method. Every command struct includes a `correlation_id: Uuid` field.
- Every `StoredEvent` carries both `correlation_id` (the originating request) and `causation_id` (the immediate cause, typically the correlation ID or a parent event ID).
- Command handlers thread the correlation ID from the command into every event they produce.

### HTTP Layer

- The `X-Correlation-ID` HTTP header carries the correlation ID between clients and the backend.
- If the header is absent, the backend generates a new v4 UUID.
- The orchestration handlers (play loop, branching) extract the correlation ID once and thread it through all downstream command invocations.

### Web Client (SvelteKit)

- The `hooks.server.ts` request handler generates a correlation ID for every incoming request and stores it in `event.locals.correlationId`.
- The server-side API client (`$lib/server/api/client.ts`) attaches the correlation ID to all outbound requests via the `X-Correlation-ID` header.
- OpenTelemetry spans include the correlation ID as a span attribute.

### iOS Client (SwiftUI)

- `CorrelationId.generate()` creates a new UUID per API request.
- `HTTPClient` injects the correlation ID into the `X-Correlation-ID` header on every request.
- `os.Logger` entries include the correlation ID for local debugging.

## Consequences

### Easier

- Any event can be traced back to its originating user action across all bounded contexts.
- OpenTelemetry spans, application logs, and event store queries can all filter by correlation ID.
- The play loop's composite response includes all event IDs, and each event's `correlation_id` confirms they belong to the same logical operation.

### More Difficult

- Every command struct must include `correlation_id` — this is boilerplate but enforced by the trait.
- Clients must generate or forward correlation IDs; forgetting to do so silently degrades traceability (the backend falls back to a generated ID, breaking the client-to-server trace).
- Testing requires threading correlation IDs through test fixtures, adding setup overhead.

### Unchanged

- The correlation ID is metadata only — it does not affect domain logic or state transitions.
- Bounded context isolation is preserved — contexts do not read each other's correlation IDs.
