import Foundation

/// View model for the narrative session list screen.
@Observable
@MainActor
final class NarrativeSessionListViewModel {
    private(set) var sessions: [NarrativeSessionSummary] = []
    private(set) var isLoading = false
    private(set) var error: APIError?

    private let endpoint: NarrativeEndpoint

    init(endpoint: NarrativeEndpoint) {
        self.endpoint = endpoint
    }

    func loadSessions() async {
        isLoading = true
        error = nil
        do {
            sessions = try await endpoint.listSessions()
        } catch let apiError as APIError {
            error = apiError
        } catch {
            self.error = .network(error.localizedDescription)
        }
        isLoading = false
    }

    func archiveSession(id: UUID) async {
        error = nil
        do {
            _ = try await endpoint.archiveSession(id: id)
            await loadSessions()
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
