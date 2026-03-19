import Foundation
import Testing

@testable import OtherworldsApp

@Suite("PlayEndpoint — correct paths and bodies via mock")
struct PlayEndpointTests {

    private func makeEndpoint() -> (PlayEndpoint, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = PlayEndpoint(client: mock)
        return (endpoint, mock)
    }

    private func responseJSON(correlationId: UUID, resolutionId: UUID) -> Data {
        """
        {
            "correlation_id": "\(correlationId)",
            "resolution_id": "\(resolutionId)",
            "intent_event_ids": [],
            "check_event_ids": [],
            "effects_event_ids": [],
            "world_state_event_ids": [],
            "narrative_event_ids": []
        }
        """.data(using: .utf8)!
    }

    @Test func resolveAction_callsCorrectPathAndMethod() async throws {
        let (endpoint, mock) = makeEndpoint()
        mock.nextResult = .success(responseJSON(correlationId: UUID(), resolutionId: UUID()))

        let request = ResolveActionRequest(
            sessionId: UUID(),
            worldId: UUID(),
            actionType: "skill_check",
            skill: nil,
            targetId: nil,
            difficultyClass: 15,
            modifier: 0,
            effects: []
        )
        let _ = try await endpoint.resolveAction(request: request)

        #expect(mock.calls.count == 1)
        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/play/resolve-action")
    }

    @Test func resolveAction_decodes_response() async throws {
        let (endpoint, mock) = makeEndpoint()
        let correlationId = UUID()
        let resolutionId = UUID()
        mock.nextResult = .success(responseJSON(correlationId: correlationId, resolutionId: resolutionId))

        let request = ResolveActionRequest(
            sessionId: UUID(),
            worldId: UUID(),
            actionType: "attack",
            skill: "strength",
            targetId: UUID(),
            difficultyClass: 12,
            modifier: 3,
            effects: [EffectSpec(effectType: "damage", targetId: nil, payload: ["amount": "5"])]
        )
        let response = try await endpoint.resolveAction(request: request)

        #expect(response.correlationId == correlationId)
        #expect(response.resolutionId == resolutionId)
        #expect(response.intentEventIds.isEmpty)
        #expect(response.checkEventIds.isEmpty)
        #expect(response.effectsEventIds.isEmpty)
        #expect(response.worldStateEventIds.isEmpty)
        #expect(response.narrativeEventIds.isEmpty)
    }
}
