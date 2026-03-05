import Foundation

/// Detailed view of a single campaign run.
///
/// Corresponds to GET /api/v1/session/:id response.
struct CampaignRunDetail: Codable, Equatable, Identifiable, Sendable {
    let runId: UUID
    let campaignId: UUID?
    let checkpointIds: [UUID]
    let version: Int

    var id: UUID { runId }
}
