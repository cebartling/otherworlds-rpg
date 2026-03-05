import Foundation

/// API client for the Inventory bounded context.
///
/// Routes are nested under /api/v1/inventory on the backend.
struct InventoryEndpoint: Sendable {
    private static let base = "/api/v1/inventory"

    private let client: HTTPClientProtocol

    init(client: HTTPClientProtocol) {
        self.client = client
    }

    /// GET /api/v1/inventory — list all inventories.
    func listInventories() async throws -> [InventorySummary] {
        try await client.get(path: Self.base, correlationId: nil)
    }

    /// GET /api/v1/inventory/:id — get inventory detail.
    func getInventory(id: UUID) async throws -> InventoryDetail {
        try await client.get(path: "\(Self.base)/\(id)", correlationId: nil)
    }

    /// POST /api/v1/inventory/add-item
    func addItem(request: AddItemRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/add-item", body: request, correlationId: nil)
    }

    /// POST /api/v1/inventory/remove-item
    func removeItem(request: RemoveItemRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/remove-item", body: request, correlationId: nil)
    }

    /// POST /api/v1/inventory/equip-item
    func equipItem(request: EquipItemRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/equip-item", body: request, correlationId: nil)
    }

    /// DELETE /api/v1/inventory/:id — archive an inventory.
    func archiveInventory(id: UUID) async throws -> CommandResponse {
        try await client.delete(path: "\(Self.base)/\(id)", correlationId: nil)
    }
}
