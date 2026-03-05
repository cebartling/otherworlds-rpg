import Foundation

/// View model for the resolution detail screen.
@Observable
@MainActor
final class ResolutionDetailViewModel {
    private(set) var resolution: ResolutionDetail?
    private(set) var isLoading = false
    private(set) var error: APIError?

    let resolutionId: UUID
    private let endpoint: RulesEndpoint

    init(resolutionId: UUID, endpoint: RulesEndpoint) {
        self.resolutionId = resolutionId
        self.endpoint = endpoint
    }

    func loadResolution() async {
        isLoading = true
        error = nil
        do {
            resolution = try await endpoint.getResolution(id: resolutionId)
        } catch let apiError as APIError {
            error = apiError
        } catch {
            self.error = .network(error.localizedDescription)
        }
        isLoading = false
    }

    func resolveCheck() async {
        error = nil
        do {
            let request = ResolveCheckRequest(resolutionId: resolutionId)
            _ = try await endpoint.resolveCheck(request: request)
            await loadResolution()
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
