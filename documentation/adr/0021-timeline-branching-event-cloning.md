# ADR-0021: Timeline Branching via Event Cloning and UUID Rewriting

## Status

Accepted

## Context

The Otherworlds RPG engine supports timeline branching — creating a divergent campaign run from a checkpoint in an existing run. Because bounded contexts are fully isolated (ADR-0004), branching a session must also clone the event histories of aggregates from other contexts (narrative sessions, inventories, world snapshots, etc.) that were registered with the source campaign run.

Two problems arise during cross-context event cloning:

1. **Identity collision** — cloned events reference the source aggregate ID throughout their payloads. If these IDs are not rewritten, the cloned aggregate's events would refer to the wrong aggregate, breaking referential integrity.
2. **Aggregate registration** — the branched campaign run needs to know which cloned aggregates belong to it, but the session context has no knowledge of other domain crates.

ADR-0014 establishes that cross-context coordination happens exclusively in the API layer. The branching strategy must follow this same principle.

## Decision

We implement timeline branching as a three-layer concern:

### 1. Context-Agnostic Event Cloning (`otherworlds-core::branching`)

A generic `clone_events_for_branch` function in the core crate handles event cloning without any domain-specific knowledge:

- Accepts source events, source aggregate ID, new aggregate ID, correlation ID, start sequence, Clock, and RNG.
- Generates fresh event IDs via `DeterministicRng`.
- Assigns new timestamps via `Clock`.
- Rewrites sequence numbers starting from `start_sequence`.
- **Recursively walks JSON payloads** and replaces any string value matching the source aggregate UUID with the new aggregate UUID. This handles nested objects and arrays without requiring domain-specific knowledge of payload structure.

### 2. Aggregate Registration (`otherworlds-session`)

The session context gains a `RegisterAggregate` command that records a `(context_name, aggregate_id)` pair on a `CampaignRun`. This allows the session aggregate to track which aggregates from other bounded contexts belong to this run — without depending on those crates. The `registered_aggregates` map is populated by the API layer after creating or cloning aggregates.

### 3. Branch Orchestration (`otherworlds-api::orchestration::branch`)

The API layer's `orchestrate_branch_timeline` function coordinates the full branch:

1. Branch the session context (replays source events up to checkpoint, produces `TimelineBranched`).
2. Load the source run's `registered_aggregates`.
3. For each registered aggregate: load its events, clone them with new IDs via `clone_events_for_branch`, and persist as a new aggregate.
4. Register each cloned aggregate with the branched run via `RegisterAggregate`.

## Consequences

### Easier

- Branching is fully generic — any bounded context's aggregates can participate without the branching code knowing their domain model.
- The UUID rewriting approach handles arbitrarily nested payloads without requiring per-event-type logic.
- New contexts automatically participate in branching by registering their aggregates with the campaign run.

### More Difficult

- UUID rewriting is a blunt instrument — it replaces all occurrences of the source aggregate ID in payloads, which could theoretically cause false positives if the same UUID appears in an unrelated field. In practice, aggregate IDs are unique enough that this is not a concern.
- The branching orchestrator must acquire the RNG mutex twice per aggregate (once for the new aggregate ID, once for event cloning), adding lock contention in theory.
- Testing requires `MultiAggregateEventRepository` to simulate multiple aggregate event streams in a single test.

### Unchanged

- Bounded context isolation is preserved — no domain crate depends on another.
- The determinism contract is maintained — Clock and RNG are injected.
- Individual context endpoints remain available for direct aggregate creation.
