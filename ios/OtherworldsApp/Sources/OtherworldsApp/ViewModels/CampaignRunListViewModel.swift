import Foundation

/// View model for the campaign run list screen.
@Observable
@MainActor
final class CampaignRunListViewModel {
    private(set) var campaignRuns: [CampaignRunSummary] = []
    private(set) var isLoading = false
    private(set) var error: APIError?

    private let endpoint: SessionEndpoint

    init(endpoint: SessionEndpoint) {
        self.endpoint = endpoint
    }

    func loadCampaignRuns() async {
        AppLogger.ui.info("Loading campaign runs...")
        isLoading = true
        error = nil
        do {
            campaignRuns = try await endpoint.listCampaignRuns()
            AppLogger.ui.info("Loaded \(self.campaignRuns.count) campaign runs")
        } catch let apiError as APIError {
            AppLogger.ui.error("Failed to load campaign runs: \(apiError.localizedDescription)")
            error = apiError
        } catch {
            AppLogger.ui.error("Failed to load campaign runs: \(error.localizedDescription)")
            self.error = .network(error.localizedDescription)
        }
        isLoading = false
    }

    func archiveCampaignRun(id: UUID) async {
        error = nil
        do {
            _ = try await endpoint.archiveCampaignRun(id: id)
            await loadCampaignRuns()
        } catch let apiError as APIError {
            AppLogger.ui.error("Failed to archive campaign run \(id): \(apiError.localizedDescription)")
            error = apiError
        } catch {
            AppLogger.ui.error("Failed to archive campaign run \(id): \(error.localizedDescription)")
            self.error = .network(error.localizedDescription)
        }
    }

    func dismissError() {
        error = nil
    }
}
