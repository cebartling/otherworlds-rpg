import Foundation

/// Summary view for listing campaigns.
///
/// Corresponds to GET /api/v1/content response items.
struct CampaignSummary: Codable, Equatable, Identifiable, Sendable {
    let campaignId: UUID
    let versionHash: String?
    let phase: String
    let version: Int

    var id: UUID { campaignId }
}
