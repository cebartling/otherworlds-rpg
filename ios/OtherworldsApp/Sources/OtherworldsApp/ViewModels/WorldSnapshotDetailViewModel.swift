import Foundation

/// View model for the world snapshot detail screen.
@Observable
@MainActor
final class WorldSnapshotDetailViewModel {
    private(set) var worldSnapshot: WorldSnapshotDetail?
    private(set) var isLoading = false
    private(set) var error: APIError?

    let worldId: UUID
    private let endpoint: WorldStateEndpoint

    init(worldId: UUID, endpoint: WorldStateEndpoint) {
        self.worldId = worldId
        self.endpoint = endpoint
    }

    func loadWorldSnapshot() async {
        AppLogger.ui.info("Loading world snapshot \(self.worldId)...")
        isLoading = true
        error = nil
        do {
            worldSnapshot = try await endpoint.getWorldSnapshot(id: worldId)
        } catch let apiError as APIError {
            AppLogger.ui.error("Failed to load world snapshot \(self.worldId): \(apiError.localizedDescription)")
            error = apiError
        } catch {
            AppLogger.ui.error("Failed to load world snapshot \(self.worldId): \(error.localizedDescription)")
            self.error = .network(error.localizedDescription)
        }
        isLoading = false
    }

    func applyEffect(factKey: String) async {
        error = nil
        do {
            let request = ApplyEffectRequest(worldId: worldId, factKey: factKey)
            _ = try await endpoint.applyEffect(request: request)
            await loadWorldSnapshot()
        } catch let apiError as APIError {
            error = apiError
        } catch {
            self.error = .network(error.localizedDescription)
        }
    }

    func setFlag(flagKey: String, value: Bool) async {
        error = nil
        do {
            let request = SetFlagRequest(worldId: worldId, flagKey: flagKey, value: value)
            _ = try await endpoint.setFlag(request: request)
            await loadWorldSnapshot()
        } catch let apiError as APIError {
            error = apiError
        } catch {
            self.error = .network(error.localizedDescription)
        }
    }

    func updateDisposition(entityId: UUID) async {
        error = nil
        do {
            let request = UpdateDispositionRequest(worldId: worldId, entityId: entityId)
            _ = try await endpoint.updateDisposition(request: request)
            await loadWorldSnapshot()
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
