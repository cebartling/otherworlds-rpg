# ADR-0008: Workspace Lint Policy (Forbid Unsafe, Clippy Pedantic)

## Status

Accepted

## Context

The Otherworlds backend is a Cargo workspace with ten crates. Without a consistent lint policy, each crate could adopt different quality standards — some allowing `unsafe` code, some ignoring Clippy warnings, some using inconsistent style. This creates a fragmented codebase where quality varies by module and unsafe behaviors can slip in undetected.

The project's determinism guarantee ([ADR-0003](0003-determinism-via-injected-abstractions.md)) and event sourcing model ([ADR-0001](0001-event-sourcing-and-cqrs.md)) demand high correctness standards. Undefined behavior from unsafe code could silently corrupt event streams or produce unreplayable state.

## Decision

We define lint policy at the workspace level in `backend/Cargo.toml` and require all crates to inherit it.

### Workspace-level configuration

```toml
[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
```

### Per-crate inheritance

Every crate's `Cargo.toml` includes:

```toml
[lints]
workspace = true
```

This ensures all crates share the same policy without duplicating configuration.

### Policy details

- **`unsafe_code = "forbid"`**: No crate may use `unsafe` blocks, `unsafe fn`, `unsafe trait`, or `unsafe impl`. This is enforced at the `forbid` level, meaning it cannot be overridden with `#[allow(unsafe_code)]` in individual crates. All dependencies that use unsafe code do so in their own crates, outside our lint scope.
- **`clippy::all = "warn"`**: Standard Clippy lints covering common mistakes, style issues, and performance pitfalls.
- **`clippy::pedantic = "warn"`**: Stricter lints that enforce idiomatic Rust patterns, including proper use of iterators, Option/Result combinators, and documentation conventions.

### Handling false positives

When a pedantic lint fires on correct, intentional code:

1. Add a targeted `#[allow(clippy::lint_name)]` on the specific item, not on the module or crate.
2. Include a brief comment explaining why the lint is suppressed.
3. Never suppress `unsafe_code` — it is `forbid` and cannot be allowed.

## Consequences

### Positive

- **No undefined behavior**: Forbidding unsafe code eliminates an entire class of memory safety bugs within the workspace. The determinism and replay guarantees depend on this.
- **Consistent quality**: All ten crates share the same lint baseline. Code review can focus on logic rather than style debates.
- **Early error detection**: Pedantic Clippy catches subtle issues (unnecessary clones, missed `must_use` attributes, suboptimal iterator usage) before they reach production.
- **Single configuration point**: Lint policy is defined once in the workspace root. Adding a new crate automatically inherits the policy.

### Negative

- **False positive noise**: `clippy::pedantic` occasionally flags valid code patterns. Developers must spend time evaluating and selectively suppressing these.
- **Onboarding friction**: Contributors unfamiliar with pedantic Clippy may find the strictness initially frustrating.
- **Third-party limitations**: Some patterns encouraged by external libraries may conflict with pedantic lints, requiring targeted `#[allow]` annotations.

### Constraints imposed

- `unsafe` code is absolutely prohibited in workspace crates. If a feature genuinely requires unsafe, it must be implemented in an external dependency, not in the workspace.
- All new crates must include `[lints] workspace = true` in their `Cargo.toml`.
- Lint suppressions must be item-level with explanatory comments, never module-level or crate-level.
