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
        isLoading = true
        error = nil
        do {
            worldSnapshots = try await endpoint.listWorldSnapshots()
        } catch let apiError as APIError {
            error = apiError
        } catch {
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
            error = apiError
        } catch {
            self.error = .network(error.localizedDescription)
        }
    }

    func dismissError() {
        error = nil
    }
}
