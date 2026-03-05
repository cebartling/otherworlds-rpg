import Foundation

/// View model for the inventory detail screen.
@Observable
@MainActor
final class InventoryDetailViewModel {
    private(set) var inventory: InventoryDetail?
    private(set) var isLoading = false
    private(set) var error: APIError?

    let inventoryId: UUID
    private let endpoint: InventoryEndpoint

    init(inventoryId: UUID, endpoint: InventoryEndpoint) {
        self.inventoryId = inventoryId
        self.endpoint = endpoint
    }

    func loadInventory() async {
        AppLogger.ui.info("Loading inventory \(self.inventoryId)...")
        isLoading = true
        error = nil
        do {
            inventory = try await endpoint.getInventory(id: inventoryId)
        } catch let apiError as APIError {
            AppLogger.ui.error("Failed to load inventory \(self.inventoryId): \(apiError.localizedDescription)")
            error = apiError
        } catch {
            AppLogger.ui.error("Failed to load inventory \(self.inventoryId): \(error.localizedDescription)")
            self.error = .network(error.localizedDescription)
        }
        isLoading = false
    }

    func addItem(name: String) async {
        error = nil
        do {
            let request = AddItemRequest(inventoryId: inventoryId, itemName: name)
            _ = try await endpoint.addItem(request: request)
            await loadInventory()
        } catch let apiError as APIError {
            error = apiError
        } catch {
            self.error = .network(error.localizedDescription)
        }
    }

    func removeItem(name: String) async {
        error = nil
        do {
            let request = RemoveItemRequest(inventoryId: inventoryId, itemName: name)
            _ = try await endpoint.removeItem(request: request)
            await loadInventory()
        } catch let apiError as APIError {
            error = apiError
        } catch {
            self.error = .network(error.localizedDescription)
        }
    }

    func equipItem(name: String) async {
        error = nil
        do {
            let request = EquipItemRequest(inventoryId: inventoryId, itemName: name)
            _ = try await endpoint.equipItem(request: request)
            await loadInventory()
        } catch let apiError as APIError {
            error = apiError
        } catch {
            self.error = .network(error.localizedDescription)
        }
    }

    func dismissError() {
        error = nil
    }
}
