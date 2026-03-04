import Foundation

/// API client for the Narrative Orchestration bounded context.
///
/// Routes are nested under /api/v1/narrative on the backend.
struct NarrativeEndpoint: Sendable {
    private static let base = "/api/v1/narrative"

    private let client: HTTPClientProtocol

    init(client: HTTPClientProtocol) {
        self.client = client
    }

    /// GET /api/v1/narrative — list all sessions.
    func listSessions() async throws -> [NarrativeSessionSummary] {
        try await client.get(path: Self.base, correlationId: nil)
    }

    /// GET /api/v1/narrative/:id — get session detail.
    func getSession(id: UUID) async throws -> NarrativeSessionView {
        try await client.get(path: "\(Self.base)/\(id)", correlationId: nil)
    }

    /// POST /api/v1/narrative/advance-beat
    func advanceBeat(request: AdvanceBeatRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/advance-beat", body: request, correlationId: nil)
    }

    /// POST /api/v1/narrative/present-choice
    func presentChoice(request: PresentChoiceRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/present-choice", body: request, correlationId: nil)
    }

    /// POST /api/v1/narrative/enter-scene
    func enterScene(request: EnterSceneRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/enter-scene", body: request, correlationId: nil)
    }

    /// POST /api/v1/narrative/select-choice
    func selectChoice(request: SelectChoiceRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/select-choice", body: request, correlationId: nil)
    }

    /// DELETE /api/v1/narrative/:id — archive a session.
    func archiveSession(id: UUID) async throws -> CommandResponse {
        try await client.delete(path: "\(Self.base)/\(id)", correlationId: nil)
    }
}
