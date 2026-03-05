# ADR-0017: iOS Vertical Slice Bounded Context Organization

## Status

Accepted

## Context

The backend organizes code into seven bounded contexts (ADR-0004), each a separate Rust crate with no cross-context dependencies. The iOS client must present UI for all seven contexts plus settings. Without an explicit organizational strategy, iOS code tends to grow into a flat collection of files where model, networking, view model, and view code intermingle, making it difficult to maintain bounded context isolation.

The iOS client needs a file organization strategy that mirrors the backend's modular structure while following SwiftUI and Swift Package conventions.

## Decision

Each bounded context is implemented as a vertical slice with a consistent set of files spanning all layers from models through views. The standard file set per context is:

- **Models** (3 files): `{Context}Summary.swift`, `{Context}Detail.swift` (or `{Context}View.swift` for the API response), and request/response types.
- **Endpoint** (1 file): `{Context}Endpoint.swift` — a `Sendable` struct wrapping `HTTPClientProtocol`.
- **View Models** (2 files): `{Context}ListViewModel.swift` and `{Context}DetailViewModel.swift`.
- **Views** (3 files): `{Context}ListView.swift`, `{Context}DetailView.swift`, and `{Context}RowView.swift`.
- **Tests** (4 files): `{Context}EndpointTests.swift`, `{Context}ListViewModelTests.swift`, `{Context}DetailViewModelTests.swift`, and view snapshot or UI tests as needed.

Directory structure follows Swift Package Manager conventions:

```
Sources/OtherworldsApp/
  Models/{Context}/           # Domain models per context
  API/Endpoints/              # One endpoint struct per context
  ViewModels/                 # List + Detail VMs per context
  Views/{Context}/            # List + Detail + Row views per context
  Views/AppTabView.swift      # Root navigation

Tests/OtherworldsAppTests/
  Endpoints/                  # Endpoint tests
  ViewModels/                 # ViewModel tests
  Mocks/                      # Shared test doubles
```

Navigation uses a `TabView` with 8 tabs, one per context plus Settings. Each tab contains its own `NavigationStack`, ensuring contexts remain isolated at the navigation level:

```swift
TabView {
    Tab("Narratives", systemImage: "book.pages") { ... }
    Tab("Characters", systemImage: "person.3") { ... }
    Tab("Inventory", systemImage: "bag.fill") { ... }
    Tab("Sessions", systemImage: "play.rectangle.fill") { ... }
    Tab("World", systemImage: "globe") { ... }
    Tab("Rules", systemImage: "dice.fill") { ... }
    Tab("Content", systemImage: "doc.text.fill") { ... }
    Tab("Settings", systemImage: "gear") { ... }
}
```

## Consequences

### Positive

- Bounded context isolation is visible in the file system — adding a new context means creating a known set of files without touching existing contexts.
- Summary/Detail model pairs match the backend's list vs. detail API response patterns, avoiding over-fetching on list screens.
- Each tab's independent `NavigationStack` prevents navigation state from leaking across contexts.
- New developers can learn one context's vertical slice and apply the same pattern to all others.
- Test coverage follows the same structure — every endpoint and view model has a corresponding test file.

### Negative

- The rigid file-per-layer convention produces many small files (~13 per context, ~91 total for 7 contexts), which can feel verbose for simple contexts.
- No cross-context imports means shared UI patterns (e.g., error banners, loading indicators) must live in a shared `Components/` directory rather than being imported from another context.
- 8 tabs may be too many for smaller phone screens — future iterations may need to consolidate or use a different navigation pattern.

### Constraints

- No context directory may import types from another context directory. Shared types live in `Models/Shared/` or `Components/`.
- Every new bounded context must follow the full vertical slice template. Do not partially implement a context.
- `AppTabView` is the sole place where contexts are composed — individual context views must not reference each other.
