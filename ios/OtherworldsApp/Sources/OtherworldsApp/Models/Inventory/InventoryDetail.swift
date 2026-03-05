import Foundation

/// Detailed view of a single inventory.
///
/// Corresponds to GET /api/v1/inventory/:id response.
struct InventoryDetail: Codable, Equatable, Identifiable, Sendable {
    let inventoryId: UUID
    let items: [String]
    let version: Int

    var id: UUID { inventoryId }
}
