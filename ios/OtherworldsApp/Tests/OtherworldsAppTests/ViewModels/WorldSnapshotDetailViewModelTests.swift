import Foundation
import Testing

@testable import OtherworldsApp

@Suite("WorldSnapshotDetailViewModel — loading, commands, error states")
@MainActor
struct WorldSnapshotDetailViewModelTests {

    private func makeViewModel(worldId: UUID = UUID()) -> (WorldSnapshotDetailViewModel, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = WorldStateEndpoint(client: mock)
        let vm = WorldSnapshotDetailViewModel(worldId: worldId, endpoint: endpoint)
        return (vm, mock)
    }

    @Test func loadWorldSnapshot_success_populatesWorldSnapshot() async {
        let worldId = UUID()
        let (vm, mock) = makeViewModel(worldId: worldId)
        let json = """
            {"world_id":"\(worldId)","facts":["Gate is open"],"flags":{"quest":true},"disposition_entity_ids":[],"version":2}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.loadWorldSnapshot()

        #expect(vm.worldSnapshot != nil)
        #expect(vm.worldSnapshot?.facts == ["Gate is open"])
        #expect(vm.worldSnapshot?.flags == ["quest": true])
        #expect(vm.isLoading == false)
        #expect(vm.error == nil)
    }

    @Test func loadWorldSnapshot_error_setsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 404))

        await vm.loadWorldSnapshot()

        #expect(vm.worldSnapshot == nil)
        #expect(vm.error == .httpError(statusCode: 404))
    }

    @Test func applyEffect_success_reloadsWorldSnapshot() async {
        let (vm, mock) = makeViewModel()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.applyEffect(factKey: "gate_opened")

        #expect(mock.calls.count == 2)
        #expect(mock.calls[0].method == "POST")
    }

    @Test func setFlag_success_reloadsWorldSnapshot() async {
        let (vm, mock) = makeViewModel()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.setFlag(flagKey: "quest_active", value: true)

        #expect(mock.calls.count == 2)
        #expect(mock.calls[0].method == "POST")
    }

    @Test func updateDisposition_success_reloadsWorldSnapshot() async {
        let (vm, mock) = makeViewModel()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.updateDisposition(entityId: UUID())

        #expect(mock.calls.count == 2)
        #expect(mock.calls[0].method == "POST")
    }

    @Test func dismissError_clearsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 500))
        await vm.loadWorldSnapshot()

        vm.dismissError()

        #expect(vm.error == nil)
    }
}
