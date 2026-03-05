import Foundation
import Testing

@testable import OtherworldsApp

@Suite("CampaignRunDetailViewModel — loading, commands, error states")
@MainActor
struct CampaignRunDetailViewModelTests {

    private func makeViewModel(runId: UUID = UUID()) -> (CampaignRunDetailViewModel, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = SessionEndpoint(client: mock)
        let vm = CampaignRunDetailViewModel(runId: runId, endpoint: endpoint)
        return (vm, mock)
    }

    @Test func loadCampaignRun_success_populatesCampaignRun() async {
        let runId = UUID()
        let (vm, mock) = makeViewModel(runId: runId)
        let cpId = UUID()
        let json = """
            {"run_id":"\(runId)","campaign_id":null,"checkpoint_ids":["\(cpId)"],"version":2}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.loadCampaignRun()

        #expect(vm.campaignRun != nil)
        #expect(vm.campaignRun?.checkpointIds == [cpId])
        #expect(vm.isLoading == false)
        #expect(vm.error == nil)
    }

    @Test func loadCampaignRun_error_setsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 404))

        await vm.loadCampaignRun()

        #expect(vm.campaignRun == nil)
        #expect(vm.error == .httpError(statusCode: 404))
    }

    @Test func createCheckpoint_success_reloadsCampaignRun() async {
        let (vm, mock) = makeViewModel()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.createCheckpoint(checkpointId: UUID())

        #expect(mock.calls.count == 2)
        #expect(mock.calls[0].method == "POST")
    }

    @Test func branchTimeline_success_reloadsCampaignRun() async {
        let (vm, mock) = makeViewModel()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.branchTimeline(fromCheckpointId: UUID(), newRunId: UUID())

        #expect(mock.calls.count == 2)
        #expect(mock.calls[0].method == "POST")
    }

    @Test func dismissError_clearsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 500))
        await vm.loadCampaignRun()

        vm.dismissError()

        #expect(vm.error == nil)
    }
}
