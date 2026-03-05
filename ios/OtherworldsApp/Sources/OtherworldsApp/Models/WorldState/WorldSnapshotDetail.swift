import Foundation

/// Detailed view of a single world snapshot.
///
/// Corresponds to GET /api/v1/world-state/:id response.
struct WorldSnapshotDetail: Codable, Equatable, Identifiable, Sendable {
    let worldId: UUID
    let facts: [String]
    let flags: [String: Bool]
    let dispositionEntityIds: [UUID]
    let version: Int

    var id: UUID { worldId }
}
