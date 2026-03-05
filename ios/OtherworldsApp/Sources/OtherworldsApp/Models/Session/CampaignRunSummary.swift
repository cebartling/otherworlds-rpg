import Foundation

/// Summary view for listing campaign runs.
///
/// Corresponds to GET /api/v1/session response items.
struct CampaignRunSummary: Codable, Equatable, Identifiable, Sendable {
    let runId: UUID
    let campaignId: UUID?
    let checkpointCount: Int
    let version: Int

    var id: UUID { runId }
}
