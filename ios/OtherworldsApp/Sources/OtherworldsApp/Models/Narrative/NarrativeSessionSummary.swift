import Foundation

/// Summary view for listing narrative sessions.
///
/// Corresponds to GET /api/v1/narrative response items.
struct NarrativeSessionSummary: Codable, Equatable, Identifiable, Sendable {
    let sessionId: UUID
    let currentBeatId: UUID?
    let currentSceneId: String?
    let version: Int

    var id: UUID { sessionId }
}
