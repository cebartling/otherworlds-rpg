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
        isLoading = true
        error = nil
        do {
            campaignRuns = try await endpoint.listCampaignRuns()
        } catch let apiError as APIError {
            error = apiError
        } catch {
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
            error = apiError
        } catch {
            self.error = .network(error.localizedDescription)
        }
    }

    func dismissError() {
        error = nil
    }
}
