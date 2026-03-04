# ADR-0014: Cross-Context Orchestration via API-Layer Play Loop

## Status

Accepted

## Context

The Otherworlds RPG engine has seven fully implemented bounded contexts, each operating in complete isolation per ADR-0004. The manifestos describe an end-to-end play loop:

```
PlayerIntentSubmitted → IntentResolved → EffectsProduced → WorldFactsChanged → BeatAdvanced
```

Currently, a client must manually call each context's API endpoints in sequence to achieve this flow. There is no single endpoint that orchestrates the full resolution cycle. This creates two problems:

1. **Client complexity** — Every client (web, iOS) must implement the same multi-step orchestration logic, including error handling and rollback.
2. **Correlation fragmentation** — Without a single orchestrator generating the correlation ID, the event chain across contexts cannot be traced as a single logical operation.

ADR-0004 explicitly identifies `otherworlds-api` as the sole integration point and lists "API-level orchestration" as the primary cross-context communication strategy.

## Decision

We will add a `play` route module to `otherworlds-api` that orchestrates the manifesto's play loop as a single HTTP request. The orchestrator:

1. **Lives in the API layer only.** No domain crate gains knowledge of other domain crates. The `otherworlds-api` binary calls into multiple crate APIs within a single handler, exactly as ADR-0004 prescribes.

2. **Exposes a `POST /api/v1/play/resolve-action` endpoint** that accepts a player intent and coordinates:
   - **Rules**: Declare intent → resolve check → produce effects
   - **World State**: Apply each produced effect as a world fact
   - **Narrative**: Advance the beat to reflect the outcome

3. **Threads a single correlation ID** through all domain commands in the chain, enabling full traceability from intent to narrative advancement.

4. **Fails atomically** — if any step fails (e.g., rules validation, world state update), the entire operation returns an error. Since each context's command handler already persists events independently, partial failure leaves the system in a consistent state: events that were persisted are valid facts; the orchestrator simply reports which step failed.

5. **Returns a composite response** containing all event IDs produced across all contexts, grouped by phase, so clients can understand what happened.

## Consequences

### Easier

- Clients call one endpoint for the full play loop instead of orchestrating 3-5 sequential API calls.
- Correlation IDs trace the complete chain from intent to narrative outcome.
- New cross-context flows (e.g., inventory effects, character XP awards) can be added by extending the orchestrator without modifying domain crates.

### More Difficult

- The orchestrator handler is more complex than single-context handlers — it must handle partial failures and compose results from multiple domain calls.
- Testing requires mock repositories that serve events for multiple aggregate types within a single test.
- If future requirements demand true transactional atomicity (all-or-nothing across contexts), we would need a saga pattern or distributed transaction — this ADR's approach accepts eventual consistency.

### Unchanged

- Domain crate isolation is preserved — no crate gains new dependencies.
- Individual context endpoints remain available for direct access.
- The determinism contract is maintained — all RNG and Clock injections flow through the same `AppState`.
