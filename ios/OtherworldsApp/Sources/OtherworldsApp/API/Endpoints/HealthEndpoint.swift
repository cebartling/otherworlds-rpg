import Foundation

/// API client for the health check endpoint.
struct HealthEndpoint: Sendable {
    private let client: HTTPClientProtocol

    init(client: HTTPClientProtocol) {
        self.client = client
    }

    /// GET /health
    func check() async throws -> HealthResponse {
        try await client.get(path: "/health", correlationId: nil)
    }
}
