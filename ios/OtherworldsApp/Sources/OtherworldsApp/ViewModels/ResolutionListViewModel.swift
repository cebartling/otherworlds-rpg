import Foundation

/// View model for the resolution list screen.
@Observable
@MainActor
final class ResolutionListViewModel {
    private(set) var resolutions: [ResolutionSummary] = []
    private(set) var isLoading = false
    private(set) var error: APIError?

    private let endpoint: RulesEndpoint

    init(endpoint: RulesEndpoint) {
        self.endpoint = endpoint
    }

    func loadResolutions() async {
        AppLogger.ui.info("Loading resolutions...")
        isLoading = true
        error = nil
        do {
            resolutions = try await endpoint.listResolutions()
            AppLogger.ui.info("Loaded \(self.resolutions.count) resolutions")
        } catch let apiError as APIError {
            AppLogger.ui.error("Failed to load resolutions: \(apiError.localizedDescription)")
            error = apiError
        } catch {
            AppLogger.ui.error("Failed to load resolutions: \(error.localizedDescription)")
            self.error = .network(error.localizedDescription)
        }
        isLoading = false
    }

    func archiveResolution(id: UUID) async {
        error = nil
        do {
            _ = try await endpoint.archiveResolution(id: id)
            await loadResolutions()
        } catch let apiError as APIError {
            AppLogger.ui.error("Failed to archive resolution \(id): \(apiError.localizedDescription)")
            error = apiError
        } catch {
            AppLogger.ui.error("Failed to archive resolution \(id): \(error.localizedDescription)")
            self.error = .network(error.localizedDescription)
        }
    }

    func dismissError() {
        error = nil
    }
}
