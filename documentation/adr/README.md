# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for Otherworlds RPG.

## What is an ADR?

An Architecture Decision Record captures a significant architectural decision along with its context and consequences. We use [Michael Nygard's lightweight format](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions) with four sections: Status, Context, Decision, and Consequences.

## When to write an ADR

Write an ADR when you make a decision that:

- Affects the structure of the system or its bounded contexts
- Constrains future implementation choices
- Would be difficult or costly to reverse
- Involves trade-offs that future contributors should understand

## ADR lifecycle

- **Proposed** — Under discussion, not yet accepted.
- **Accepted** — Approved and in effect.
- **Deprecated** — No longer applies; superseded by a later ADR.
- **Superseded** — Replaced by another ADR (link to successor).

## Index

| ADR | Title | Status |
|-----|-------|--------|
| [0001](0001-event-sourcing-and-cqrs.md) | Event Sourcing and CQRS | Accepted |
| [0002](0002-modular-monolith-architecture.md) | Modular Monolith Architecture | Accepted |
| [0003](0003-determinism-via-injected-abstractions.md) | Determinism via Injected Abstractions | Accepted |
| [0004](0004-bounded-context-isolation.md) | Bounded Context Isolation via Strict Dependency Direction | Accepted |
| [0005](0005-campaign-independence.md) | Externally-Authored Versioned Campaigns | Accepted |
| [0006](0006-rust-axum-postgresql-tech-stack.md) | Rust, Axum, and PostgreSQL Tech Stack | Accepted |
| [0007](0007-api-route-versioning.md) | API Route Nesting and Versioning | Accepted |
| [0008](0008-workspace-lint-policy.md) | Workspace Lint Policy (Forbid Unsafe, Clippy Pedantic) | Accepted |
| [0009](0009-structured-domain-error-handling.md) | Structured Domain Error Handling with thiserror | Accepted |
| [0010](0010-editorconfig-formatting-standards.md) | Cross-Language Formatting Standards via .editorconfig | Accepted |
