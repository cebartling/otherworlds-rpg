import Foundation

/// API client for the Content bounded context.
///
/// Routes are nested under /api/v1/content on the backend.
struct ContentEndpoint: Sendable {
    private static let base = "/api/v1/content"

    private let client: HTTPClientProtocol

    init(client: HTTPClientProtocol) {
        self.client = client
    }

    /// GET /api/v1/content — list all campaigns.
    func listCampaigns() async throws -> [CampaignSummary] {
        try await client.get(path: Self.base, correlationId: nil)
    }

    /// GET /api/v1/content/:id — get campaign detail.
    func getCampaign(id: UUID) async throws -> CampaignDetail {
        try await client.get(path: "\(Self.base)/\(id)", correlationId: nil)
    }

    /// POST /api/v1/content/ingest
    func ingestCampaign(request: IngestCampaignRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/ingest", body: request, correlationId: nil)
    }

    /// POST /api/v1/content/validate
    func validateCampaign(request: ValidateCampaignRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/validate", body: request, correlationId: nil)
    }

    /// POST /api/v1/content/compile
    func compileCampaign(request: CompileCampaignRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/compile", body: request, correlationId: nil)
    }

    /// DELETE /api/v1/content/:id — archive a campaign.
    func archiveCampaign(id: UUID) async throws -> CommandResponse {
        try await client.delete(path: "\(Self.base)/\(id)", correlationId: nil)
    }
}
