import Foundation

/// View model for the campaign list screen.
@Observable
@MainActor
final class CampaignListViewModel {
    private(set) var campaigns: [CampaignSummary] = []
    private(set) var isLoading = false
    private(set) var error: APIError?

    private let endpoint: ContentEndpoint

    init(endpoint: ContentEndpoint) {
        self.endpoint = endpoint
    }

    func loadCampaigns() async {
        isLoading = true
        error = nil
        do {
            campaigns = try await endpoint.listCampaigns()
        } catch let apiError as APIError {
            error = apiError
        } catch {
            self.error = .network(error.localizedDescription)
        }
        isLoading = false
    }

    func archiveCampaign(id: UUID) async {
        error = nil
        do {
            _ = try await endpoint.archiveCampaign(id: id)
            await loadCampaigns()
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
