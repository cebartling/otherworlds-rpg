import Foundation
import Testing

@testable import OtherworldsApp

@Suite("NarrativeSessionDetailViewModel — loading, success, error states")
@MainActor
struct NarrativeSessionDetailViewModelTests {

    private func makeViewModel(sessionId: UUID = UUID()) -> (NarrativeSessionDetailViewModel, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = NarrativeEndpoint(client: mock)
        let vm = NarrativeSessionDetailViewModel(sessionId: sessionId, endpoint: endpoint)
        return (vm, mock)
    }

    private func sessionJSON(id: UUID) -> Data {
        """
        {
            "session_id":"\(id)",
            "current_beat_id":null,
            "choice_ids":[],
            "current_scene_id":"tavern",
            "scene_history":["intro","tavern"],
            "active_choice_options":[
                {"label":"Talk","target_scene_id":"chat"}
            ],
            "version":2
        }
        """.data(using: .utf8)!
    }

    @Test func loadSession_success_populatesSession() async {
        let sessionId = UUID()
        let (vm, mock) = makeViewModel(sessionId: sessionId)
        mock.nextResult = .success(sessionJSON(id: sessionId))

        await vm.loadSession()

        #expect(vm.session != nil)
        #expect(vm.session?.sessionId == sessionId)
        #expect(vm.session?.currentSceneId == "tavern")
        #expect(vm.session?.activeChoiceOptions.count == 1)
        #expect(vm.isLoading == false)
        #expect(vm.error == nil)
    }

    @Test func loadSession_error_setsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.server(statusCode: 404, error: "not_found", message: "Not found"))

        await vm.loadSession()

        #expect(vm.session == nil)
        #expect(vm.error == .server(statusCode: 404, error: "not_found", message: "Not found"))
    }

    @Test func advanceBeat_callsEndpointAndReloads() async {
        let sessionId = UUID()
        let (vm, mock) = makeViewModel(sessionId: sessionId)
        // advanceBeat POST returns CommandResponse, then loadSession GET returns session
        mock.nextResult = .success("""
            {"event_ids":[]}
            """.data(using: .utf8)!)

        await vm.advanceBeat()

        #expect(mock.calls.count >= 1)
        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path.contains("advance-beat"))
    }

    @Test func selectChoice_callsEndpointWithCorrectBody() async {
        let sessionId = UUID()
        let (vm, mock) = makeViewModel(sessionId: sessionId)
        mock.nextResult = .success("""
            {"event_ids":[]}
            """.data(using: .utf8)!)

        let target = TargetSceneRequest(
            sceneId: "cave",
            narrativeText: "Dark.",
            choices: [],
            npcRefs: nil
        )
        await vm.selectChoice(index: 1, targetScene: target)

        #expect(mock.calls.count >= 1)
        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path.contains("select-choice"))
    }

    @Test func dismissError_clearsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 500))

        await vm.loadSession()
        #expect(vm.error != nil)

        vm.dismissError()
        #expect(vm.error == nil)
    }
}
