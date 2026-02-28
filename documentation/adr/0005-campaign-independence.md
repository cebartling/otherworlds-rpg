# ADR-0005: Externally-Authored Versioned Campaigns

## Status

Accepted

## Context

Otherworlds RPG separates three concepts: the **engine** (deterministic runtime), the **campaign** (authored world content), and the **run** (recorded playthrough history). The product manifesto states that "engine and campaign are independent" and that "campaigns are versioned, external, and loadable."

If campaign content is embedded in the engine codebase, several problems arise:

- Content changes require engine redeployment.
- Content authors must work within the engine's development workflow.
- Saved campaign runs become tightly coupled to specific engine builds, making backward compatibility fragile.
- Testing engine mechanics independently of any specific campaign becomes difficult.

The system needs a clean boundary between the engine (which enforces laws) and campaigns (which define possibility).

## Decision

Campaigns are externally authored, compiled, versioned, and hash-validated. The engine treats campaigns as immutable, versioned data packages.

### Authoring

- Campaigns are authored in a Markdown-based format outside the engine codebase.
- Content includes scene definitions, beat structures, NPC data, item catalogs, quest graphs, and world facts.
- The Content Authoring bounded context (`otherworlds-content`) handles ingestion, validation, and compilation of raw campaign files into a runtime-ready format.

### Versioning and hashing

- Every compiled campaign has a unique **version hash** derived from its content.
- Campaign metadata includes a campaign ID, version hash, and compatibility constraints (minimum/maximum engine versions).
- Content changes produce a new version hash. Previous versions remain valid and accessible.

### Saved campaign runs

A saved campaign run stores:

- **Campaign ID** — Which campaign was played.
- **Campaign version hash** — The exact content version used.
- **Engine version** — The engine version that produced the run.
- **Event stream** — The complete ordered history of domain events.

### Compatibility guarantee

A campaign update must not silently invalidate prior runs. When loading a saved run:

1. The engine checks that the campaign version hash matches the run's recorded hash.
2. If the campaign has been updated, the engine warns the player and may refuse to load if the changes are incompatible.
3. Replaying a run always uses the campaign version recorded in the run, not the latest version.

## Consequences

### Positive

- **Independent evolution**: Engine and content evolve on separate release cycles. Content updates do not require engine redeployment, and engine updates do not require content re-authoring.
- **Content portability**: Campaigns can be shared, forked, and distributed as standalone packages.
- **Run integrity**: Saved runs are always valid against the campaign version they were created with, even as the campaign evolves.
- **Clear separation of concerns**: Engine developers focus on mechanics and infrastructure; content authors focus on narrative and world design.

### Negative

- **Compilation pipeline**: Requires building and maintaining a campaign compiler that transforms Markdown-based sources into a validated runtime format.
- **Version management complexity**: The system must track campaign versions, engine compatibility ranges, and run-to-campaign bindings.
- **Storage overhead**: Multiple campaign versions may need to be retained to support old saved runs.

### Constraints imposed

- Campaign content must never be hardcoded in engine source files.
- The `otherworlds-content` crate must validate and compile campaigns without depending on other domain crates.
- Saved runs must record the exact campaign version hash and engine version used.
- Engine changes that alter event semantics must increment a compatibility version.
