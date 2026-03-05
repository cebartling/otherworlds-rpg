import Foundation

/// View model for the campaign run detail screen.
@Observable
@MainActor
final class CampaignRunDetailViewModel {
    private(set) var campaignRun: CampaignRunDetail?
    private(set) var isLoading = false
    private(set) var error: APIError?

    let runId: UUID
    private let endpoint: SessionEndpoint

    init(runId: UUID, endpoint: SessionEndpoint) {
        self.runId = runId
        self.endpoint = endpoint
    }

    func loadCampaignRun() async {
        AppLogger.ui.info("Loading campaign run \(self.runId)...")
        isLoading = true
        error = nil
        do {
            campaignRun = try await endpoint.getCampaignRun(id: runId)
        } catch let apiError as APIError {
            AppLogger.ui.error("Failed to load campaign run \(self.runId): \(apiError.localizedDescription)")
            error = apiError
        } catch {
            AppLogger.ui.error("Failed to load campaign run \(self.runId): \(error.localizedDescription)")
            self.error = .network(error.localizedDescription)
        }
        isLoading = false
    }

    func createCheckpoint(checkpointId: UUID) async {
        error = nil
        do {
            let request = CreateCheckpointRequest(runId: runId, checkpointId: checkpointId)
            _ = try await endpoint.createCheckpoint(request: request)
            await loadCampaignRun()
        } catch let apiError as APIError {
            error = apiError
        } catch {
            self.error = .network(error.localizedDescription)
        }
    }

    func branchTimeline(fromCheckpointId: UUID, newRunId: UUID) async {
        error = nil
        do {
            let request = BranchTimelineRequest(
                runId: runId,
                fromCheckpointId: fromCheckpointId,
                newRunId: newRunId
            )
            _ = try await endpoint.branchTimeline(request: request)
            await loadCampaignRun()
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
