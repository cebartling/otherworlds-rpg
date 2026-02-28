# ADR-0011: Identity UUID Generation Exempt from Determinism Contract

## Status

Accepted

## Context

ADR-0003 establishes that all domain logic must use injected `Clock` and `DeterministicRng` abstractions — no implicit randomness. However, throughout the codebase, `Uuid::new_v4()` is used to generate identity values:

- **Event IDs** (`event_id`) in aggregate domain methods (`aggregates.rs`)
- **Entity IDs** (`beat_id`, `choice_id`) in domain event payloads
- **Correlation IDs** (`correlation_id`) generated server-side in API route handlers

These calls bypass the injected `DeterministicRng` and use the system's cryptographic RNG directly. This appears to violate the determinism constraint.

The question: does this actually break replay determinism?

## Decision

Identity UUID generation via `Uuid::new_v4()` is **exempt** from the determinism contract. This applies to:

- `event_id` — unique identifier for a stored event
- `correlation_id` — traces a command through its effect chain
- `causation_id` — links an effect to its cause
- Entity identifiers generated during command handling (`beat_id`, `choice_id`, etc.)

### Rationale

1. **Replay operates on stored events, not re-executed commands.** The determinism guarantee states: "replaying a campaign run with the same seed and event stream produces identical state." Events are append-only facts. When replaying, the system loads and re-applies *already-persisted* events (which contain their original UUIDs). It does not re-execute commands to regenerate new UUIDs.

2. **Identity values do not determine state transitions.** A `beat_id` of `abc-123` vs `def-456` produces the same aggregate state after `apply()`. The determinism contract cares about *outcomes* (did the skill check pass? which branch did the narrative take?), not about the specific UUIDs assigned to events.

3. **The `DeterministicRng` trait is designed for mechanical outcomes.** Its methods (`next_u32_range`, `next_f64`) map to dice rolls, probability checks, and loot tables — values that directly affect game state. UUID generation is a different concern (addressing/identity) that does not belong on this trait.

4. **Threading RNG through every ID generation site adds complexity for zero behavioral benefit.** Every aggregate method and handler would need an additional `&mut dyn DeterministicRng` parameter solely to generate identifiers that have no impact on deterministic replay.

### Boundary

This exemption does **not** extend to:

- Random values that affect state transitions (skill checks, damage rolls, loot selection) — these must use `DeterministicRng`
- Timestamps — these must use the injected `Clock`
- Any value that changes the outcome of `apply()` or command handler logic

## Consequences

### Positive

- Keeps domain method signatures clean — no RNG parameter needed for identity generation
- Aligns with how replay actually works (re-apply stored events, not re-execute commands)
- Eliminates false-positive determinism violations in code review

### Negative

- Two identical command invocations with the same inputs will produce events with different `event_id` and `correlation_id` values — this is acceptable because the *state transitions* are identical
- Developers must understand the distinction between "identity randomness" (exempt) and "mechanical randomness" (must use `DeterministicRng`)
