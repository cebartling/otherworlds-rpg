import Foundation
import Testing

@testable import OtherworldsApp

@Suite("ResolutionDetailViewModel — loading, commands, error states")
@MainActor
struct ResolutionDetailViewModelTests {

    private func makeViewModel(resolutionId: UUID = UUID()) -> (ResolutionDetailViewModel, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = RulesEndpoint(client: mock)
        let vm = ResolutionDetailViewModel(resolutionId: resolutionId, endpoint: endpoint)
        return (vm, mock)
    }

    @Test func loadResolution_success_populatesResolution() async {
        let resolutionId = UUID()
        let (vm, mock) = makeViewModel(resolutionId: resolutionId)
        let json = """
            {"resolution_id":"\(resolutionId)","phase":"IntentDeclared","intent":null,"check_result":null,"effects":[],"version":2}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.loadResolution()

        #expect(vm.resolution != nil)
        #expect(vm.resolution?.phase == "IntentDeclared")
        #expect(vm.isLoading == false)
        #expect(vm.error == nil)
    }

    @Test func loadResolution_error_setsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 404))

        await vm.loadResolution()

        #expect(vm.resolution == nil)
        #expect(vm.error == .httpError(statusCode: 404))
    }

    @Test func resolveCheck_success_reloadsResolution() async {
        let (vm, mock) = makeViewModel()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.resolveCheck()

        #expect(mock.calls.count == 2)
        #expect(mock.calls[0].method == "POST")
    }

    @Test func dismissError_clearsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 500))
        await vm.loadResolution()

        vm.dismissError()

        #expect(vm.error == nil)
    }
}
