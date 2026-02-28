# Otherworlds RPG

## Technical Manifesto

------------------------------------------------------------------------

Otherworlds RPG is a deterministic, event-driven narrative engine built
with modern systems design principles.

This document defines the architectural commitments behind the engine.

It is not a roadmap.\
It is a set of constraints.

------------------------------------------------------------------------

# 1. Architectural Philosophy

Otherworlds is built on five technical pillars:

1.  Deterministic Core
2.  Bounded Context Isolation
3.  Event-Driven State Evolution
4.  Modular Monolith First
5.  Cloud-Native Deployment

Every implementation decision must align with these.

------------------------------------------------------------------------

# 2. Core Technology Stack

## Backend

-   **Language:** Rust
-   **Web Framework:** Axum
-   **Runtime:** Tokio
-   **Architecture:** Modular monolith
-   **Containerization:** Docker
-   **Deployment Target:** Digital Ocean
-   **Database:** PostgreSQL 18
-   **Object Storage:** Cloud object storage (for image assets)
-   **Observability:** OpenTelemetry-compatible tracing
-   **API Style:** JSON over HTTP (initially), event-driven internally

------------------------------------------------------------------------

## Frontend Strategy

Multiple frontends will interact with the same backend API.

### Native iOS Client

-   Swift
-   SwiftUI
-   REST/JSON API integration
-   Eventually real-time updates via WebSockets (optional)

### Web Client

-   SvelteKit
-   Server-side rendering for SEO-friendly campaign content
-   Typed API integration
-   Optional progressive enhancement

The backend is the source of truth.\
Frontends are consumers.

------------------------------------------------------------------------

# 3. Bounded Context Architecture

Otherworlds is divided into explicit bounded contexts.

These are enforced at the crate/module level in Rust.

------------------------------------------------------------------------

## 3.1 Core Contexts

### Narrative Orchestration

**Responsibility:** - Scene progression - Beat advancement - Choice
presentation - Quest state transitions

**Does Not:** - Roll dice - Apply world mutations - Persist directly

------------------------------------------------------------------------

### Rules & Resolution

**Responsibility:** - Interpret player intent - Perform checks (skill,
combat, etc.) - Generate deterministic outcomes - Produce effects

**Does Not:** - Advance narrative - Modify world state directly

------------------------------------------------------------------------

### World State

**Responsibility:** - Canonical world facts - Flags - Dispositions -
Locations - Relationships - Time state

World state evolves only via explicit effects.

------------------------------------------------------------------------

### Character Management

**Responsibility:** - Character sheets - Attributes and skills - Status
effects - Experience and leveling

------------------------------------------------------------------------

## 3.2 Supporting Contexts

### Inventory & Economy

-   Items
-   Containers
-   Transactions
-   Equipment

### Session & Progress

-   Campaign runs
-   Save/load
-   Checkpoints
-   Replay
-   Branching timelines

### Content Authoring

-   Markdown-based campaign ingestion
-   Validation
-   Compilation into runtime format
-   Versioning and hashing

------------------------------------------------------------------------

# 4. Event-Driven Architecture

Otherworlds is event-driven internally.

All meaningful state changes:

-   Begin as commands
-   Produce domain events
-   Result in explicit effects
-   Mutate state only through effect application

Example chain:

PlayerIntentSubmitted\
→ IntentResolved\
→ EffectsProduced\
→ WorldFactsChanged\
→ BeatAdvanced

Events are append-only facts.

------------------------------------------------------------------------

## 4.1 Event Store

-   Backed by PostgreSQL 18
-   Append-only stream per Campaign Run
-   Optimistic concurrency control
-   Correlation and causation IDs
-   Versioned schemas

Event history is the source of truth.

------------------------------------------------------------------------

# 5. Modular Monolith First

Otherworlds begins as a modular monolith.

## Why

-   Faster iteration
-   Simpler deployment
-   Lower operational overhead
-   Stronger invariants across contexts

The codebase is structured as:

-   Cargo workspace
-   One binary
-   Multiple domain crates
-   Clear dependency direction

Internal modules communicate via in-process message passing and traits.

------------------------------------------------------------------------

## 5.1 Future Microservice Evolution

If scale requires it:

-   Contexts may be extracted into services
-   Event streams can be externalized
-   Messaging infrastructure can be introduced

But:

Premature distribution is prohibited.

------------------------------------------------------------------------

# 6. Persistence Model

## PostgreSQL 18

Used for:

-   Event store
-   Projections (read models)
-   Campaign metadata
-   User accounts (if introduced)

Schema evolution must be explicit and versioned.

No hidden migrations.

------------------------------------------------------------------------

## Object Storage

Used for:

-   Campaign artwork
-   Image assets
-   Possibly compiled campaign packs

The database stores metadata and references only.

------------------------------------------------------------------------

# 7. Deployment Model

## Containerization

-   Docker images
-   Multi-stage builds
-   Minimal runtime footprint

## Cloud Environment

-   Digital Ocean App Platform or Kubernetes
-   Managed PostgreSQL
-   Managed object storage
-   Environment-configured secrets

The system must be deployable with:

-   Stateless application containers
-   Externalized database
-   Externalized storage

------------------------------------------------------------------------

# 8. Determinism Guarantee

Otherworlds must remain deterministic.

This requires:

-   Injected clock abstraction
-   Injected RNG abstraction
-   No implicit randomness
-   No hidden time-based side effects
-   No reliance on non-deterministic AI output for mechanics

AI may enhance narrative description.

It does not determine state transitions.

------------------------------------------------------------------------

# 9. Campaign Independence

Campaigns are:

-   Authored externally (Markdown-based)
-   Compiled
-   Versioned
-   Hash-validated

A saved Campaign Run stores:

-   Campaign ID
-   Campaign version hash
-   Engine version
-   Event stream

A campaign update must not silently invalidate prior runs.

------------------------------------------------------------------------

# 10. API Design

The backend exposes:

-   Command endpoints
-   Query endpoints
-   Possibly WebSocket streams (future)

Commands are explicit.

Queries read projections.

State mutation never occurs in query handlers.

------------------------------------------------------------------------

# 11. Observability & Debuggability

Every request:

-   Has a correlation ID
-   Is traceable across command chain
-   Can reconstruct cause/effect relationships

Replays must be possible in development mode.

Debugging a campaign run should be equivalent to replaying its event
history.

------------------------------------------------------------------------

# 12. Non-Negotiable Constraints

Otherworlds must remain:

-   Deterministic
-   Event-driven
-   Modular
-   Replayable
-   Versioned
-   Infrastructure-agnostic

If a proposed feature:

-   Couples campaign logic to engine code
-   Introduces implicit state mutation
-   Breaks replay determinism
-   Requires hidden side effects

It must be rejected or redesigned.

------------------------------------------------------------------------

# 13. Long-Term Technical Vision

Otherworlds is not just a game backend.

It is:

-   A deterministic narrative runtime
-   A domain-driven systems experiment
-   A replayable simulation engine
-   A platform for multiple worlds

Each campaign is an Otherworld.

Each run is a history.

The backend enforces law.

The frontends present experience.

The event log preserves truth.
