import Foundation

/// API client for the Rules bounded context.
///
/// Routes are nested under /api/v1/rules on the backend.
struct RulesEndpoint: Sendable {
    private static let base = "/api/v1/rules"

    private let client: HTTPClientProtocol

    init(client: HTTPClientProtocol) {
        self.client = client
    }

    /// GET /api/v1/rules — list all resolutions.
    func listResolutions() async throws -> [ResolutionSummary] {
        try await client.get(path: Self.base, correlationId: nil)
    }

    /// GET /api/v1/rules/:id — get resolution detail.
    func getResolution(id: UUID) async throws -> ResolutionDetail {
        try await client.get(path: "\(Self.base)/\(id)", correlationId: nil)
    }

    /// POST /api/v1/rules/declare-intent
    func declareIntent(request: DeclareIntentRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/declare-intent", body: request, correlationId: nil)
    }

    /// POST /api/v1/rules/resolve-check
    func resolveCheck(request: ResolveCheckRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/resolve-check", body: request, correlationId: nil)
    }

    /// POST /api/v1/rules/produce-effects
    func produceEffects(request: ProduceEffectsRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/produce-effects", body: request, correlationId: nil)
    }

    /// DELETE /api/v1/rules/:id — archive a resolution.
    func archiveResolution(id: UUID) async throws -> CommandResponse {
        try await client.delete(path: "\(Self.base)/\(id)", correlationId: nil)
    }
}
