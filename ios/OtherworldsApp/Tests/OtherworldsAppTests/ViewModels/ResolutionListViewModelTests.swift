import Foundation
import Testing

@testable import OtherworldsApp

@Suite("ResolutionListViewModel — loading, success, error states")
@MainActor
struct ResolutionListViewModelTests {

    private func makeViewModel() -> (ResolutionListViewModel, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = RulesEndpoint(client: mock)
        let vm = ResolutionListViewModel(endpoint: endpoint)
        return (vm, mock)
    }

    @Test func loadResolutions_success_populatesResolutions() async {
        let (vm, mock) = makeViewModel()
        let resolutionId = UUID()
        let json = """
            [{"resolution_id":"\(resolutionId)","phase":"Created","outcome":null,"version":1}]
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.loadResolutions()

        #expect(vm.resolutions.count == 1)
        #expect(vm.resolutions[0].resolutionId == resolutionId)
        #expect(vm.resolutions[0].phase == "Created")
        #expect(vm.isLoading == false)
        #expect(vm.error == nil)
    }

    @Test func loadResolutions_error_setsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 500))

        await vm.loadResolutions()

        #expect(vm.resolutions.isEmpty)
        #expect(vm.isLoading == false)
        #expect(vm.error == .httpError(statusCode: 500))
    }

    @Test func archiveResolution_success_reloadsResolutions() async {
        let (vm, mock) = makeViewModel()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        await vm.archiveResolution(id: UUID())

        #expect(mock.calls.count == 2)
    }

    @Test func dismissError_clearsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 404))
        await vm.loadResolutions()

        vm.dismissError()

        #expect(vm.error == nil)
    }
}
