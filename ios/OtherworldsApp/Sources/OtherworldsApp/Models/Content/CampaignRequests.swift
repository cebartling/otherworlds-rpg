import Foundation

/// Request to ingest a campaign from source.
struct IngestCampaignRequest: Codable, Equatable, Sendable {
    let campaignId: UUID
    let source: String
}

/// Request to validate a campaign.
struct ValidateCampaignRequest: Codable, Equatable, Sendable {
    let campaignId: UUID
}

/// Request to compile a campaign.
struct CompileCampaignRequest: Codable, Equatable, Sendable {
    let campaignId: UUID
}
