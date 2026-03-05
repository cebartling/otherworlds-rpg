import Foundation

/// View model for the world snapshot list screen.
@Observable
@MainActor
final class WorldSnapshotListViewModel {
    private(set) var worldSnapshots: [WorldSnapshotSummary] = []
    private(set) var isLoading = false
    private(set) var error: APIError?

    private let endpoint: WorldStateEndpoint

    init(endpoint: WorldStateEndpoint) {
        self.endpoint = endpoint
    }

    func loadWorldSnapshots() async {
        AppLogger.ui.info("Loading world snapshots...")
        isLoading = true
        error = nil
        do {
            worldSnapshots = try await endpoint.listWorldSnapshots()
            AppLogger.ui.info("Loaded \(self.worldSnapshots.count) world snapshots")
        } catch let apiError as APIError {
            AppLogger.ui.error("Failed to load world snapshots: \(apiError.localizedDescription)")
            error = apiError
        } catch {
            AppLogger.ui.error("Failed to load world snapshots: \(error.localizedDescription)")
            self.error = .network(error.localizedDescription)
        }
        isLoading = false
    }

    func archiveWorldSnapshot(id: UUID) async {
        error = nil
        do {
            _ = try await endpoint.archiveWorldSnapshot(id: id)
            await loadWorldSnapshots()
        } catch let apiError as APIError {
            AppLogger.ui.error("Failed to archive world snapshot \(id): \(apiError.localizedDescription)")
            error = apiError
        } catch {
            AppLogger.ui.error("Failed to archive world snapshot \(id): \(error.localizedDescription)")
            self.error = .network(error.localizedDescription)
        }
    }

    func dismissError() {
        error = nil
    }
}
