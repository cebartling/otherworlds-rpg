import Foundation

/// Summary view for listing world snapshots.
///
/// Corresponds to GET /api/v1/world-state response items.
struct WorldSnapshotSummary: Codable, Equatable, Identifiable, Sendable {
    let worldId: UUID
    let factCount: Int
    let flagCount: Int
    let version: Int

    var id: UUID { worldId }
}
