import Foundation
import Testing

@testable import OtherworldsApp

@Suite("RulesEndpoint — correct paths and bodies via mock")
struct RulesEndpointTests {

    private func makeEndpoint() -> (RulesEndpoint, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = RulesEndpoint(client: mock)
        return (endpoint, mock)
    }

    @Test func listResolutions_callsCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        mock.nextResult = .success("[]".data(using: .utf8)!)

        let _: [ResolutionSummary] = try await endpoint.listResolutions()

        #expect(mock.calls.count == 1)
        #expect(mock.calls[0].method == "GET")
        #expect(mock.calls[0].path == "/api/v1/rules")
    }

    @Test func getResolution_includesIdInPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let resolutionId = UUID()
        let json = """
            {"resolution_id":"\(resolutionId)","phase":"Created","intent":null,"check_result":null,"effects":[],"version":1}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let _ = try await endpoint.getResolution(id: resolutionId)

        #expect(mock.calls[0].path == "/api/v1/rules/\(resolutionId)")
        #expect(mock.calls[0].method == "GET")
    }

    @Test func declareIntent_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = DeclareIntentRequest(
            resolutionId: UUID(),
            intentId: UUID(),
            actionType: "attack",
            skill: nil,
            targetId: nil,
            difficultyClass: 10,
            modifier: 0
        )
        let _ = try await endpoint.declareIntent(request: request)

        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/rules/declare-intent")
    }

    @Test func resolveCheck_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = ResolveCheckRequest(resolutionId: UUID())
        let _ = try await endpoint.resolveCheck(request: request)

        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/rules/resolve-check")
    }

    @Test func produceEffects_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = ProduceEffectsRequest(
            resolutionId: UUID(),
            effects: [EffectEntry(targetId: UUID(), effectType: "damage", magnitude: 5)]
        )
        let _ = try await endpoint.produceEffects(request: request)

        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/rules/produce-effects")
    }

    @Test func archiveResolution_deletesCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let resolutionId = UUID()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let _ = try await endpoint.archiveResolution(id: resolutionId)

        #expect(mock.calls[0].method == "DELETE")
        #expect(mock.calls[0].path == "/api/v1/rules/\(resolutionId)")
    }
}
