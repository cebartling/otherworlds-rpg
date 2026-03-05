# ADR-0016: iOS Protocol-Based HTTP Abstraction

## Status

Accepted

## Context

The iOS client must communicate with the Rust backend over JSON HTTP. Network I/O is inherently side-effectful and non-deterministic — exactly the kind of dependency the backend isolates behind trait abstractions (`EventRepository`, `Clock`, `DeterministicRng` per ADR-0003). The iOS client needs an equivalent testability seam so that view models and endpoints can be tested without real network calls.

Options considered: direct `URLSession` usage in endpoints (no abstraction), a third-party HTTP client like Alamofire, or a protocol-based abstraction with a production `URLSession` implementation and a mock for tests.

## Decision

All network I/O flows through `HTTPClientProtocol`, a generic Swift protocol that defines three methods matching the backend's REST verbs:

```swift
protocol HTTPClientProtocol: Sendable {
    func get<T: Decodable & Sendable>(path: String, correlationId: String?) async throws -> T
    func post<T: Decodable & Sendable, B: Encodable & Sendable>(
        path: String,
        body: B,
        correlationId: String?
    ) async throws -> T
    func delete<T: Decodable & Sendable>(path: String, correlationId: String?) async throws -> T
}
```

Two implementations exist:

1. **`HTTPClient`** — Production implementation backed by `URLSession`. Configures `JSONDecoder` with `convertFromSnakeCase` and `JSONEncoder` with `convertToSnakeCase` to match the Rust backend's serde conventions. Marked `@unchecked Sendable` because `URLSession` is thread-safe but not formally `Sendable`.

2. **`MockHTTPClient`** — Test double that records every call (method, path, correlation ID, body) into a `calls` array and returns a pre-configured `Result<Data, APIError>`. This allows tests to assert both that the correct requests were made and that the view model handles success/failure correctly.

Endpoint structs are value types (`struct`, `Sendable`) that accept `HTTPClientProtocol` via `init` and delegate all I/O to it:

```swift
struct CharacterEndpoint: Sendable {
    private static let base = "/api/v1/characters"
    private let client: HTTPClientProtocol

    init(client: HTTPClientProtocol) {
        self.client = client
    }

    func listCharacters() async throws -> [CharacterSummary] {
        try await client.get(path: Self.base, correlationId: nil)
    }
}
```

## Consequences

### Positive

- Mirrors the backend's trait-based dependency injection, creating a consistent architectural philosophy across platforms.
- View model tests run in milliseconds with no network dependency.
- `MockHTTPClient` records calls, enabling assertion on request paths, methods, and correlation IDs.
- Correlation ID threading is built into the protocol from the start, matching the backend's `X-Correlation-ID` header pattern.
- No third-party HTTP library dependency — uses `URLSession` directly.

### Negative

- The protocol's generic constraints (`Decodable & Sendable`) add verbosity compared to Alamofire-style convenience APIs.
- `@unchecked Sendable` on `HTTPClient` is a concession to `URLSession` not formally conforming to `Sendable`, though it is documented as thread-safe.
- Adding new HTTP methods (e.g., `PUT`, `PATCH`) requires extending both the protocol and all implementations.

### Constraints

- All network access must go through `HTTPClientProtocol`. Direct `URLSession` usage outside `HTTPClient` is prohibited.
- Endpoint structs must be `Sendable` value types, not classes.
- JSON coding strategies must use snake_case conversion to match the Rust backend's serde defaults.
