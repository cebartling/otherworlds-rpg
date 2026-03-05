import Foundation
import Testing

@testable import OtherworldsApp

@Suite("CampaignRunListViewModel — loading, success, error states")
@MainActor
struct CampaignRunListViewModelTests {

    private func makeViewModel() -> (CampaignRunListViewModel, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = SessionEndpoint(client: mock)
        let vm = CampaignRunListViewModel(endpoint: endpoint)
        return (vm, mock)
    }

    @Test func loadCampaignRuns_success_populatesCampaignRuns() async {
        let (vm, mock) = makeViewModel()
        let runId = UUID()
        let json = """
            [{"run_id":"\(runId)","campaign_id":null,"checkpoint_count":2,"version":1}]
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.loadCampaignRuns()

        #expect(vm.campaignRuns.count == 1)
        #expect(vm.campaignRuns[0].runId == runId)
        #expect(vm.campaignRuns[0].checkpointCount == 2)
        #expect(vm.isLoading == false)
        #expect(vm.error == nil)
    }

    @Test func loadCampaignRuns_error_setsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 500))

        await vm.loadCampaignRuns()

        #expect(vm.campaignRuns.isEmpty)
        #expect(vm.isLoading == false)
        #expect(vm.error == .httpError(statusCode: 500))
    }

    @Test func archiveCampaignRun_success_reloadsCampaignRuns() async {
        let (vm, mock) = makeViewModel()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.archiveCampaignRun(id: UUID())

        #expect(mock.calls.count == 2)
    }

    @Test func dismissError_clearsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 404))
        await vm.loadCampaignRuns()

        vm.dismissError()

        #expect(vm.error == nil)
    }
}
