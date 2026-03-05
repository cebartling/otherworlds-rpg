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
        AppLogger.ui.info("Loading narrative sessions...")
        isLoading = true
        error = nil
        do {
            sessions = try await endpoint.listSessions()
            AppLogger.ui.info("Loaded \(self.sessions.count) narrative sessions")
        } catch let apiError as APIError {
            AppLogger.ui.error("Failed to load narrative sessions: \(apiError.localizedDescription)")
            error = apiError
        } catch {
            AppLogger.ui.error("Failed to load narrative sessions: \(error.localizedDescription)")
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
            AppLogger.ui.error("Failed to archive narrative session \(id): \(apiError.localizedDescription)")
            error = apiError
        } catch {
            AppLogger.ui.error("Failed to archive narrative session \(id): \(error.localizedDescription)")
            self.error = .network(error.localizedDescription)
        }
    }

    func dismissError() {
        error = nil
    }
}
