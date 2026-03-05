import Foundation
import Testing

@testable import OtherworldsApp

@Suite("WorldSnapshotListViewModel — loading, success, error states")
@MainActor
struct WorldSnapshotListViewModelTests {

    private func makeViewModel() -> (WorldSnapshotListViewModel, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = WorldStateEndpoint(client: mock)
        let vm = WorldSnapshotListViewModel(endpoint: endpoint)
        return (vm, mock)
    }

    @Test func loadWorldSnapshots_success_populatesWorldSnapshots() async {
        let (vm, mock) = makeViewModel()
        let worldId = UUID()
        let json = """
            [{"world_id":"\(worldId)","fact_count":2,"flag_count":1,"version":1}]
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.loadWorldSnapshots()

        #expect(vm.worldSnapshots.count == 1)
        #expect(vm.worldSnapshots[0].worldId == worldId)
        #expect(vm.worldSnapshots[0].factCount == 2)
        #expect(vm.isLoading == false)
        #expect(vm.error == nil)
    }

    @Test func loadWorldSnapshots_error_setsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 500))

        await vm.loadWorldSnapshots()

        #expect(vm.worldSnapshots.isEmpty)
        #expect(vm.isLoading == false)
        #expect(vm.error == .httpError(statusCode: 500))
    }

    @Test func archiveWorldSnapshot_success_reloadsWorldSnapshots() async {
        let (vm, mock) = makeViewModel()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.archiveWorldSnapshot(id: UUID())

        #expect(mock.calls.count == 2)
    }

    @Test func dismissError_clearsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 404))
        await vm.loadWorldSnapshots()

        vm.dismissError()

        #expect(vm.error == nil)
    }
}
