# ADR-0007: API Route Nesting and Versioning

## Status

Accepted

## Context

The Otherworlds backend serves two frontend clients (SvelteKit web, Swift/SwiftUI iOS) over JSON HTTP. The API must:

- **Evolve without breaking existing clients** — Mobile apps in particular cannot be force-updated. API changes must be backward-compatible or versioned.
- **Reflect bounded context boundaries** — Routes should clearly indicate which domain context handles each request, reinforcing the modular architecture.
- **Separate commands from queries** — Consistent with CQRS ([ADR-0001](0001-event-sourcing-and-cqrs.md)), the API must distinguish endpoints that mutate state from those that read projections.

The alternative — flat, unversioned routes (e.g., `/characters`, `/narrative`) — provides no evolution path and no structural clarity about which context owns a resource.

## Decision

All API routes are nested under `/api/v1/{context}`, with a version prefix to support future API evolution.

### Route structure

```
/health                          → Health check (unversioned)
/api/v1/narrative                → Narrative Orchestration
/api/v1/rules                    → Rules & Resolution
/api/v1/world                   → World State
/api/v1/characters              → Character Management
/api/v1/inventory               → Inventory & Economy
/api/v1/sessions                → Session & Progress
/api/v1/content                 → Content Authoring
```

### Implementation

- Each bounded context defines its own `Router` in `src/routes/{context}.rs` within `otherworlds-api`.
- The main router in `main.rs` merges all context routers using `Router::nest("/api/v1/{context}", routes::{context}::router())`.
- The health check at `/health` returns `{ "status": "ok", "version": "..." }` and is not versioned (infrastructure concern, not API contract).

### Command vs. query endpoints

Within each context's route namespace:

- **Command endpoints** accept POST requests with command payloads and return the resulting domain events or acknowledgment.
- **Query endpoints** accept GET requests and return projected read model state.
- Commands mutate state; queries never mutate. This separation is reflected in HTTP method usage and handler implementation.

### Versioning strategy

- The `v1` prefix is part of the URL path, not a header.
- When breaking changes are required, a `v2` route tree will be introduced alongside `v1`. Both versions may coexist during migration.
- Non-breaking additions (new fields, new endpoints) do not require a version bump.

## Consequences

### Positive

- **Clear ownership**: Every route path makes it obvious which bounded context handles the request. This aids debugging, documentation, and access control.
- **Safe evolution**: The version prefix allows introducing breaking changes without disrupting existing clients. Mobile clients can migrate at their own pace.
- **CQRS alignment**: HTTP method conventions (POST for commands, GET for queries) make the command/query separation visible at the API level.
- **Composable routing**: Each context's `Router` is defined, tested, and maintained independently, then composed in the API binary.

### Negative

- **Version bump cost**: Introducing `v2` requires duplicating route definitions and potentially maintaining two versions of handlers during the transition period.
- **URL verbosity**: `/api/v1/characters/{id}/skills` is longer than `/characters/{id}/skills`, though this is a minor concern for API consumers.
- **Cross-context queries**: Queries that span multiple contexts (e.g., "character with inventory") require either multiple client requests or a dedicated aggregation endpoint in `otherworlds-api`.

### Constraints imposed

- All domain-facing routes must be nested under `/api/v1/`. No top-level domain routes.
- Each bounded context must define its own router function and must not register routes in another context's namespace.
- Health and infrastructure endpoints remain unversioned at the root level.
