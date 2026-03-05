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
| [0011](0011-identity-uuid-determinism-exemption.md) | Identity UUID Determinism Exemption | Accepted |
| [0012](0012-campaign-markdown-format.md) | Campaign Markdown Format | Accepted |
| [0013](0013-screenplay-pattern-acceptance-tests.md) | Screenplay Pattern Acceptance Tests | Accepted |
| [0014](0014-cross-context-orchestration.md) | Cross-Context Orchestration via API-Layer Play Loop | Accepted |
| [0015](0015-ios-mvvm-native-observation.md) | iOS MVVM with Native Observation | Accepted |
| [0016](0016-ios-protocol-based-http-abstraction.md) | iOS Protocol-Based HTTP Abstraction | Accepted |
| [0017](0017-ios-vertical-slice-bounded-contexts.md) | iOS Vertical Slice Bounded Contexts | Accepted |
| [0018](0018-sveltekit-server-side-loading.md) | SvelteKit Server-Side Loading | Accepted |
| [0019](0019-web-private-server-api-client.md) | Web Private Server API Client | Accepted |
| [0020](0020-cross-platform-theme-system.md) | Cross-Platform Theme System | Accepted |
| [0021](0021-timeline-branching-event-cloning.md) | Timeline Branching via Event Cloning and UUID Rewriting | Accepted |
| [0022](0022-correlation-id-propagation.md) | Correlation ID Propagation Across All Platforms | Accepted |
| [0023](0023-cross-platform-observability-opentelemetry.md) | Cross-Platform Observability via OpenTelemetry | Accepted |
| [0024](0024-proactive-optimistic-concurrency-control.md) | Proactive Optimistic Concurrency Control in Event Store | Accepted |
| [0025](0025-local-observability-infrastructure.md) | Local Observability Infrastructure via Docker Compose | Accepted |
| [0026](0026-test-support-repository-pattern.md) | Test Support Repository Pattern | Accepted |
