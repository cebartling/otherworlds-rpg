import Foundation

/// API client for the World State bounded context.
///
/// Routes are nested under /api/v1/world-state on the backend.
struct WorldStateEndpoint: Sendable {
    private static let base = "/api/v1/world-state"

    private let client: HTTPClientProtocol

    init(client: HTTPClientProtocol) {
        self.client = client
    }

    /// GET /api/v1/world-state — list all world snapshots.
    func listWorldSnapshots() async throws -> [WorldSnapshotSummary] {
        try await client.get(path: Self.base, correlationId: nil)
    }

    /// GET /api/v1/world-state/:id — get world snapshot detail.
    func getWorldSnapshot(id: UUID) async throws -> WorldSnapshotDetail {
        try await client.get(path: "\(Self.base)/\(id)", correlationId: nil)
    }

    /// POST /api/v1/world-state/apply-effect
    func applyEffect(request: ApplyEffectRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/apply-effect", body: request, correlationId: nil)
    }

    /// POST /api/v1/world-state/set-flag
    func setFlag(request: SetFlagRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/set-flag", body: request, correlationId: nil)
    }

    /// POST /api/v1/world-state/update-disposition
    func updateDisposition(request: UpdateDispositionRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/update-disposition", body: request, correlationId: nil)
    }

    /// DELETE /api/v1/world-state/:id — archive a world snapshot.
    func archiveWorldSnapshot(id: UUID) async throws -> CommandResponse {
        try await client.delete(path: "\(Self.base)/\(id)", correlationId: nil)
    }
}
