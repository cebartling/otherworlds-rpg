import Foundation

/// View model for the Play tab — submits a resolve-action request and holds the response.
@Observable
@MainActor
final class ResolveActionViewModel {
    private(set) var result: ResolveActionResponse?
    private(set) var isLoading = false
    private(set) var error: APIError?

    private let endpoint: PlayEndpoint

    init(endpoint: PlayEndpoint) {
        self.endpoint = endpoint
    }

    func resolveAction(request: ResolveActionRequest) async {
        AppLogger.ui.info("Submitting resolve-action...")
        isLoading = true
        error = nil
        do {
            result = try await endpoint.resolveAction(request: request)
        } catch let apiError as APIError {
            AppLogger.ui.error("resolve-action failed: \(apiError.localizedDescription)")
            error = apiError
        } catch {
            AppLogger.ui.error("resolve-action failed: \(error.localizedDescription)")
            self.error = .network(error.localizedDescription)
        }
        isLoading = false
    }

    func dismissError() {
        error = nil
    }
}
