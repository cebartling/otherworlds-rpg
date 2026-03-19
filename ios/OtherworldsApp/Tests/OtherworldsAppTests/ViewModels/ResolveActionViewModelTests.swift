import Foundation
import Testing

@testable import OtherworldsApp

@Suite("ResolveActionViewModel — loading, success, error states")
@MainActor
struct ResolveActionViewModelTests {

    private func makeViewModel() -> (ResolveActionViewModel, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = PlayEndpoint(client: mock)
        let vm = ResolveActionViewModel(endpoint: endpoint)
        return (vm, mock)
    }

    private func responseJSON(correlationId: UUID, resolutionId: UUID) -> Data {
        """
        {
            "correlation_id": "\(correlationId)",
            "resolution_id": "\(resolutionId)",
            "intent_event_ids": ["00000000-0000-0000-0000-000000000001"],
            "check_event_ids": ["00000000-0000-0000-0000-000000000002"],
            "effects_event_ids": ["00000000-0000-0000-0000-000000000003"],
            "world_state_event_ids": ["00000000-0000-0000-0000-000000000004"],
            "narrative_event_ids": ["00000000-0000-0000-0000-000000000005"]
        }
        """.data(using: .utf8)!
    }

    private func makeRequest() -> ResolveActionRequest {
        ResolveActionRequest(
            sessionId: UUID(),
            worldId: UUID(),
            actionType: "skill_check",
            skill: "perception",
            targetId: nil,
            difficultyClass: 15,
            modifier: 2,
            effects: []
        )
    }

    @Test func resolveAction_success_populatesResult() async {
        let correlationId = UUID()
        let resolutionId = UUID()
        let (vm, mock) = makeViewModel()
        mock.nextResult = .success(responseJSON(correlationId: correlationId, resolutionId: resolutionId))

        await vm.resolveAction(request: makeRequest())

        #expect(vm.result != nil)
        #expect(vm.result?.correlationId == correlationId)
        #expect(vm.result?.resolutionId == resolutionId)
        #expect(vm.result?.intentEventIds.count == 1)
        #expect(vm.result?.checkEventIds.count == 1)
        #expect(vm.result?.effectsEventIds.count == 1)
        #expect(vm.result?.worldStateEventIds.count == 1)
        #expect(vm.result?.narrativeEventIds.count == 1)
        #expect(vm.isLoading == false)
        #expect(vm.error == nil)
    }

    @Test func resolveAction_error_setsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.server(statusCode: 422, error: "unprocessable", message: "Invalid request"))

        await vm.resolveAction(request: makeRequest())

        #expect(vm.result == nil)
        #expect(vm.error == .server(statusCode: 422, error: "unprocessable", message: "Invalid request"))
        #expect(vm.isLoading == false)
    }

    @Test func dismissError_clearsError() async {
        let (vm, mock) = makeViewModel()
        mock.nextResult = .failure(.httpError(statusCode: 500))

        await vm.resolveAction(request: makeRequest())
        #expect(vm.error != nil)

        vm.dismissError()
        #expect(vm.error == nil)
    }
}
