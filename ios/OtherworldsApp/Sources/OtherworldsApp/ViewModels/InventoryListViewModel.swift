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
        isLoading = true
        error = nil
        do {
            inventories = try await endpoint.listInventories()
        } catch let apiError as APIError {
            error = apiError
        } catch {
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
            error = apiError
        } catch {
            self.error = .network(error.localizedDescription)
        }
    }

    func dismissError() {
        error = nil
    }
}
