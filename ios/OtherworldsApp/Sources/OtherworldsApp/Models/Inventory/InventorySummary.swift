import Foundation

/// Summary view for listing inventories.
///
/// Corresponds to GET /api/v1/inventory response items.
struct InventorySummary: Codable, Equatable, Identifiable, Sendable {
    let inventoryId: UUID
    let itemCount: Int
    let version: Int

    var id: UUID { inventoryId }
}
