import Foundation

/// View model for the campaign detail screen.
@Observable
@MainActor
final class CampaignDetailViewModel {
    private(set) var campaign: CampaignDetail?
    private(set) var isLoading = false
    private(set) var error: APIError?

    let campaignId: UUID
    private let endpoint: ContentEndpoint

    init(campaignId: UUID, endpoint: ContentEndpoint) {
        self.campaignId = campaignId
        self.endpoint = endpoint
    }

    func loadCampaign() async {
        AppLogger.ui.info("Loading campaign \(self.campaignId)...")
        isLoading = true
        error = nil
        do {
            campaign = try await endpoint.getCampaign(id: campaignId)
        } catch let apiError as APIError {
            AppLogger.ui.error("Failed to load campaign \(self.campaignId): \(apiError.localizedDescription)")
            error = apiError
        } catch {
            AppLogger.ui.error("Failed to load campaign \(self.campaignId): \(error.localizedDescription)")
            self.error = .network(error.localizedDescription)
        }
        isLoading = false
    }

    func ingestCampaign(source: String) async {
        error = nil
        do {
            let request = IngestCampaignRequest(campaignId: campaignId, source: source)
            _ = try await endpoint.ingestCampaign(request: request)
            await loadCampaign()
        } catch let apiError as APIError {
            error = apiError
        } catch {
            self.error = .network(error.localizedDescription)
        }
    }

    func validateCampaign() async {
        error = nil
        do {
            let request = ValidateCampaignRequest(campaignId: campaignId)
            _ = try await endpoint.validateCampaign(request: request)
            await loadCampaign()
        } catch let apiError as APIError {
            error = apiError
        } catch {
            self.error = .network(error.localizedDescription)
        }
    }

    func compileCampaign() async {
        error = nil
        do {
            let request = CompileCampaignRequest(campaignId: campaignId)
            _ = try await endpoint.compileCampaign(request: request)
            await loadCampaign()
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
