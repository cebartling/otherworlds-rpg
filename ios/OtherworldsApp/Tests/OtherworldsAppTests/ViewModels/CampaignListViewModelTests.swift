import Foundation
import Testing

@testable import OtherworldsApp

@Suite("CampaignListViewModel — loading, success, error states")
@MainActor
struct CampaignListViewModelTests {

    private func makeViewModel() -> (CampaignListViewModel, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = ContentEndpoint(client: mock)
        let vm = CampaignListViewModel(endpoint: endpoint)
        return (vm, mock)
    }

    @Test func loadCampaigns_success_populatesCampaigns() async {
        let (vm, mock) = makeViewModel()
        let campaignId = UUID()
        let json = """
            [{"campaign_id":"\(campaignId)","version_hash":"abc","phase":"ingested","version":1}]
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.loadCampaigns()

        #expect(vm.campaigns.count == 1)
        #expect(vm.campaigns[0].campaignId == campaignId)
        #expect(vm.campaigns[0].phase == "ingested")
        #expect(vm.isLoading == false)
        #expect(vm.error == nil)
    }

    @Test func loadCampaigns_error_setsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 500))

        await vm.loadCampaigns()

        #expect(vm.campaigns.isEmpty)
        #expect(vm.isLoading == false)
        #expect(vm.error == .httpError(statusCode: 500))
    }

    @Test func archiveCampaign_success_reloadsCampaigns() async {
        let (vm, mock) = makeViewModel()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.archiveCampaign(id: UUID())

        #expect(mock.calls.count == 2)
    }

    @Test func dismissError_clearsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 404))
        await vm.loadCampaigns()

        vm.dismissError()

        #expect(vm.error == nil)
    }
}
