import Foundation
import Testing

@testable import OtherworldsApp

@Suite("WorldStateEndpoint — correct paths and bodies via mock")
struct WorldStateEndpointTests {

    private func makeEndpoint() -> (WorldStateEndpoint, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = WorldStateEndpoint(client: mock)
        return (endpoint, mock)
    }

    @Test func listWorldSnapshots_callsCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        mock.nextResult = .success("[]".data(using: .utf8)!)

        let _: [WorldSnapshotSummary] = try await endpoint.listWorldSnapshots()

        #expect(mock.calls.count == 1)
        #expect(mock.calls[0].method == "GET")
        #expect(mock.calls[0].path == "/api/v1/world-state")
    }

    @Test func getWorldSnapshot_includesIdInPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let worldId = UUID()
        let json = """
            {"world_id":"\(worldId)","facts":[],"flags":{},"disposition_entity_ids":[],"version":1}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let _ = try await endpoint.getWorldSnapshot(id: worldId)

        #expect(mock.calls[0].path == "/api/v1/world-state/\(worldId)")
        #expect(mock.calls[0].method == "GET")
    }

    @Test func applyEffect_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = ApplyEffectRequest(worldId: UUID(), factKey: "gate_opened")
        let _ = try await endpoint.applyEffect(request: request)

        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/world-state/apply-effect")
    }

    @Test func setFlag_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = SetFlagRequest(worldId: UUID(), flagKey: "quest_active", value: true)
        let _ = try await endpoint.setFlag(request: request)

        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/world-state/set-flag")
    }

    @Test func updateDisposition_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = UpdateDispositionRequest(worldId: UUID(), entityId: UUID())
        let _ = try await endpoint.updateDisposition(request: request)

        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/world-state/update-disposition")
    }

    @Test func archiveWorldSnapshot_deletesCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let worldId = UUID()
        let json = """
            {"event_ids":["00000000-0000-0000-0000-000000000001"]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let _ = try await endpoint.archiveWorldSnapshot(id: worldId)

        #expect(mock.calls[0].method == "DELETE")
        #expect(mock.calls[0].path == "/api/v1/world-state/\(worldId)")
    }
}
