import Foundation
import Testing

@testable import OtherworldsApp

@Suite("CampaignDetailViewModel — loading, commands, error states")
@MainActor
struct CampaignDetailViewModelTests {

    private func makeViewModel(campaignId: UUID = UUID()) -> (CampaignDetailViewModel, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = ContentEndpoint(client: mock)
        let vm = CampaignDetailViewModel(campaignId: campaignId, endpoint: endpoint)
        return (vm, mock)
    }

    @Test func loadCampaign_success_populatesCampaign() async {
        let campaignId = UUID()
        let (vm, mock) = makeViewModel(campaignId: campaignId)
        let json = """
            {"campaign_id":"\(campaignId)","version_hash":"abc","source":"# Test","compiled_data":null,"phase":"ingested","version":2}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.loadCampaign()

        #expect(vm.campaign != nil)
        #expect(vm.campaign?.phase == "ingested")
        #expect(vm.campaign?.source == "# Test")
        #expect(vm.isLoading == false)
        #expect(vm.error == nil)
    }

    @Test func loadCampaign_error_setsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 404))

        await vm.loadCampaign()

        #expect(vm.campaign == nil)
        #expect(vm.error == .httpError(statusCode: 404))
    }

    @Test func ingestCampaign_success_reloadsCampaign() async {
        let (vm, mock) = makeViewModel()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.ingestCampaign(source: "# Test Campaign")

        #expect(mock.calls.count == 2)
        #expect(mock.calls[0].method == "POST")
    }

    @Test func validateCampaign_success_reloadsCampaign() async {
        let (vm, mock) = makeViewModel()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.validateCampaign()

        #expect(mock.calls.count == 2)
        #expect(mock.calls[0].method == "POST")
    }

    @Test func compileCampaign_success_reloadsCampaign() async {
        let (vm, mock) = makeViewModel()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.compileCampaign()

        #expect(mock.calls.count == 2)
        #expect(mock.calls[0].method == "POST")
    }

    @Test func dismissError_clearsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 500))
        await vm.loadCampaign()

        vm.dismissError()

        #expect(vm.error == nil)
    }
}
