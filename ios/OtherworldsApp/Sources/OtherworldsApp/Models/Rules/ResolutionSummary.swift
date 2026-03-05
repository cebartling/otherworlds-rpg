import Foundation

/// Summary view for listing resolutions.
///
/// Corresponds to GET /api/v1/rules response items.
struct ResolutionSummary: Codable, Equatable, Identifiable, Sendable {
    let resolutionId: UUID
    let phase: String
    let outcome: String?
    let version: Int

    var id: UUID { resolutionId }
}
