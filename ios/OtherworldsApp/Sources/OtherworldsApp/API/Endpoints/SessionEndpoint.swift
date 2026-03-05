import Foundation

/// API client for the Session bounded context.
///
/// Routes are nested under /api/v1/session on the backend.
struct SessionEndpoint: Sendable {
    private static let base = "/api/v1/session"

    private let client: HTTPClientProtocol

    init(client: HTTPClientProtocol) {
        self.client = client
    }

    /// GET /api/v1/session — list all campaign runs.
    func listCampaignRuns() async throws -> [CampaignRunSummary] {
        try await client.get(path: Self.base, correlationId: nil)
    }

    /// GET /api/v1/session/:id — get campaign run detail.
    func getCampaignRun(id: UUID) async throws -> CampaignRunDetail {
        try await client.get(path: "\(Self.base)/\(id)", correlationId: nil)
    }

    /// POST /api/v1/session/start
    func startCampaignRun(request: StartCampaignRunRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/start", body: request, correlationId: nil)
    }

    /// POST /api/v1/session/create-checkpoint
    func createCheckpoint(request: CreateCheckpointRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/create-checkpoint", body: request, correlationId: nil)
    }

    /// POST /api/v1/session/branch-timeline
    func branchTimeline(request: BranchTimelineRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/branch-timeline", body: request, correlationId: nil)
    }

    /// DELETE /api/v1/session/:id — archive a campaign run.
    func archiveCampaignRun(id: UUID) async throws -> CommandResponse {
        try await client.delete(path: "\(Self.base)/\(id)", correlationId: nil)
    }
}
