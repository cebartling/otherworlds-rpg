import Foundation

/// View model for the narrative session detail/play screen.
@Observable
@MainActor
final class NarrativeSessionDetailViewModel {
    private(set) var session: NarrativeSessionView?
    private(set) var isLoading = false
    private(set) var error: APIError?

    let sessionId: UUID
    private let endpoint: NarrativeEndpoint

    init(sessionId: UUID, endpoint: NarrativeEndpoint) {
        self.sessionId = sessionId
        self.endpoint = endpoint
    }

    func loadSession() async {
        AppLogger.ui.info("Loading narrative session \(self.sessionId)...")
        isLoading = true
        error = nil
        do {
            session = try await endpoint.getSession(id: sessionId)
        } catch let apiError as APIError {
            AppLogger.ui.error("Failed to load narrative session \(self.sessionId): \(apiError.localizedDescription)")
            error = apiError
        } catch {
            AppLogger.ui.error("Failed to load narrative session \(self.sessionId): \(error.localizedDescription)")
            self.error = .network(error.localizedDescription)
        }
        isLoading = false
    }

    func advanceBeat() async {
        error = nil
        do {
            _ = try await endpoint.advanceBeat(request: AdvanceBeatRequest(sessionId: sessionId))
            await loadSession()
        } catch let apiError as APIError {
            error = apiError
        } catch {
            self.error = .network(error.localizedDescription)
        }
    }

    func selectChoice(index: Int, targetScene: TargetSceneRequest) async {
        error = nil
        do {
            let request = SelectChoiceRequest(
                sessionId: sessionId,
                choiceIndex: index,
                targetScene: targetScene
            )
            _ = try await endpoint.selectChoice(request: request)
            await loadSession()
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
