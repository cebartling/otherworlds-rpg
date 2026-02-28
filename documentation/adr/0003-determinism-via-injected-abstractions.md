# ADR-0003: Determinism via Injected Abstractions

## Status

Accepted

## Context

Otherworlds RPG is a narrative engine where campaign runs must be fully replayable. Given the same inputs and seed, a run must produce identical results. Two common sources of non-determinism threaten this guarantee:

- **Time**: System clocks vary between machines and invocations. Any domain logic that reads `SystemTime::now()` directly produces unreproducible results.
- **Randomness**: Standard RNG implementations are not seedable or recordable by default. Skill checks, loot rolls, and other probabilistic outcomes must be reproducible on replay.

Additionally, the product manifesto explicitly rejects hidden mechanics and implicit state mutation. AI-generated content may enhance narrative description but must never determine mechanical outcomes (e.g., whether a skill check succeeds or fails).

## Decision

We enforce determinism by injecting two trait abstractions into all domain logic that requires time or randomness.

### Clock trait

Defined in `otherworlds-core::clock`:

- `fn now(&self) -> DateTime<Utc>` — Returns the current timestamp.
- **`SystemClock`** — Production implementation that delegates to the system clock.
- **Test implementations** — Return fixed or incrementing timestamps for reproducible tests and replays.

### DeterministicRng trait

Defined in `otherworlds-core::rng`:

- `fn next_u32(&mut self) -> u32` — Returns the next pseudorandom value.
- `fn next_range(&mut self, min: i32, max: i32) -> i32` — Returns a value in `[min, max]`.
- **`StdRng`** — Production implementation backed by `rand::rngs::StdRng`.
- **Test implementations** — Seeded or pre-programmed RNGs for deterministic test outcomes.

### Injection

- `AppState` in `otherworlds-api` holds `Arc<dyn Clock + Send + Sync>` and `Arc<Mutex<dyn DeterministicRng + Send>>`.
- Command handlers receive these traits as dependencies rather than accessing system resources directly.
- No domain crate may call `std::time::SystemTime`, `Instant::now()`, `rand::thread_rng()`, or any other non-deterministic source directly.

### AI constraint

AI (LLM) integration may generate or enhance narrative text (scene descriptions, NPC dialogue flavor). AI output must never be used to determine state transitions, check outcomes, or modify game mechanics. All mechanical resolution flows through the deterministic command-event pipeline.

## Consequences

### Positive

- **Full replayability**: Replaying a campaign run with the same seed and event stream produces identical state, enabling save/restore and time-travel debugging.
- **Testability**: Unit tests inject fixed clocks and seeded RNGs, eliminating flaky tests caused by timing or randomness.
- **Transparency**: Players and authors can inspect why an outcome occurred — no hidden randomness or time-dependent behavior.
- **Branching timelines**: Forking a run at any point and replaying from there is straightforward because all inputs are recorded.

### Negative

- **Indirection cost**: Every domain function that needs time or randomness takes an additional trait parameter, adding verbosity to function signatures.
- **Discipline required**: Developers must remember to use injected traits rather than direct system calls. Code review and linting must catch violations.
- **RNG serialization**: Recording and replaying RNG state adds complexity to the event store and replay infrastructure.

### Constraints imposed

- All domain logic must receive `Clock` and `DeterministicRng` as injected dependencies.
- Direct use of system time or thread-local RNG in domain crates is prohibited.
- AI-generated output must be treated as presentation-layer enhancement only, never as input to state transitions.
