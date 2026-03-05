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
        isLoading = true
        error = nil
        do {
            resolutions = try await endpoint.listResolutions()
        } catch let apiError as APIError {
            error = apiError
        } catch {
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
            error = apiError
        } catch {
            self.error = .network(error.localizedDescription)
        }
    }

    func dismissError() {
        error = nil
    }
}
