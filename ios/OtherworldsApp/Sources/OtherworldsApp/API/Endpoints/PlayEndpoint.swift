import Foundation

/// API client for the cross-context Play orchestration route.
///
/// Routes are nested under /api/v1/play on the backend.
struct PlayEndpoint: Sendable {
    private static let base = "/api/v1/play"

    private let client: HTTPClientProtocol

    init(client: HTTPClientProtocol) {
        self.client = client
    }

    /// POST /api/v1/play/resolve-action — orchestrate the full play loop.
    func resolveAction(request: ResolveActionRequest) async throws -> ResolveActionResponse {
        try await client.post(path: "\(Self.base)/resolve-action", body: request, correlationId: nil)
    }
}
