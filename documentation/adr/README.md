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
