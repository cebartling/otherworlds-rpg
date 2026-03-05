import Foundation

/// View model for the inventory list screen.
@Observable
@MainActor
final class InventoryListViewModel {
    private(set) var inventories: [InventorySummary] = []
    private(set) var isLoading = false
    private(set) var error: APIError?

    private let endpoint: InventoryEndpoint

    init(endpoint: InventoryEndpoint) {
        self.endpoint = endpoint
    }

    func loadInventories() async {
        AppLogger.ui.info("Loading inventories...")
        isLoading = true
        error = nil
        do {
            inventories = try await endpoint.listInventories()
            AppLogger.ui.info("Loaded \(self.inventories.count) inventories")
        } catch let apiError as APIError {
            AppLogger.ui.error("Failed to load inventories: \(apiError.localizedDescription)")
            error = apiError
        } catch {
            AppLogger.ui.error("Failed to load inventories: \(error.localizedDescription)")
            self.error = .network(error.localizedDescription)
        }
        isLoading = false
    }

    func archiveInventory(id: UUID) async {
        error = nil
        do {
            _ = try await endpoint.archiveInventory(id: id)
            await loadInventories()
        } catch let apiError as APIError {
            AppLogger.ui.error("Failed to archive inventory \(id): \(apiError.localizedDescription)")
            error = apiError
        } catch {
            AppLogger.ui.error("Failed to archive inventory \(id): \(error.localizedDescription)")
            self.error = .network(error.localizedDescription)
        }
    }

    func dismissError() {
        error = nil
    }
}
