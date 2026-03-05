import Foundation

/// Detailed view of a single campaign.
///
/// Corresponds to GET /api/v1/content/:id response.
struct CampaignDetail: Codable, Equatable, Identifiable, Sendable {
    let campaignId: UUID
    let versionHash: String?
    let source: String?
    let compiledData: String?
    let phase: String
    let version: Int

    var id: UUID { campaignId }
}
