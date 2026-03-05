# ADR-0015: iOS MVVM with Native Swift Observation

## Status

Accepted

## Context

The iOS client needs a reactive data-binding pattern for its view models. Several options exist in the Swift ecosystem: Combine's `ObservableObject` with `@Published`, third-party frameworks like RxSwift, and Swift 6's native `@Observable` macro introduced in the Observation framework. The backend already demonstrates a preference for minimal dependencies and framework-native solutions (e.g., Tokio rather than a custom runtime, `thiserror` rather than hand-rolled error types). The iOS client should follow the same philosophy.

Combine's `ObservableObject` requires `@Published` property wrappers, explicit `objectWillChange` publishers, and `@ObservedObject`/`@StateObject` in views. RxSwift adds a large third-party dependency with its own concurrency model. Swift 6's `@Observable` macro eliminates all of this boilerplate with compiler-synthesized change tracking and integrates directly with Swift's structured concurrency (`async`/`await`, `@MainActor`).

## Decision

All iOS view models use the `@Observable @MainActor final class` pattern with no reactive framework dependency.

The concrete pattern is:

```swift
@Observable
@MainActor
final class CharacterListViewModel {
    private(set) var characters: [CharacterSummary] = []
    private(set) var isLoading = false
    private(set) var error: APIError?

    private let endpoint: CharacterEndpoint

    init(endpoint: CharacterEndpoint) {
        self.endpoint = endpoint
    }

    func loadCharacters() async { /* ... */ }
}
```

Key conventions:

1. **Class declaration**: `@Observable @MainActor final class {Context}{List|Detail}ViewModel`.
2. **Three standard properties**: a domain collection or item (`characters`, `session`, etc.), `isLoading: Bool`, and `error: APIError?` — all `private(set)`.
3. **Dependency injection**: Endpoints are injected via `init`, not constructed internally.
4. **Views own their view models** via `@State private var viewModel`, not `@ObservedObject` or `@StateObject`.
5. **All public methods are `async`** — callers use `Task { await viewModel.load() }`. No Combine publishers, no `sink` subscriptions.
6. **`@MainActor` isolation** ensures all UI-bound state mutations happen on the main thread without explicit `DispatchQueue.main` calls.

## Consequences

### Positive

- Zero third-party reactive framework dependency — the Observation framework ships with Swift 6.
- Simpler mental model: properties are plain stored properties with compiler-synthesized change tracking, not wrapped publishers.
- `@MainActor` eliminates an entire class of thread-safety bugs around UI updates.
- Dependency injection via `init` makes view models trivially testable with mock endpoints.
- Consistent with the backend's philosophy of using language-native features over framework abstractions.

### Negative

- Requires Swift 6 / iOS 17+ as the minimum deployment target, excluding older devices.
- Developers familiar with Combine's publisher-chain style must adapt to the simpler async/await pattern.
- `@Observable` does not support `willSet`/`didSet`-style observation hooks that Combine's `objectWillChange` provided — if needed, manual notification would require a different approach.

### Constraints

- All new view models must follow this pattern. Do not introduce `ObservableObject`, `@Published`, or Combine imports.
- View models must remain `final class` (not struct) because `@Observable` requires reference semantics for identity-based change tracking.
