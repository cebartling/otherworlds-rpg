import Foundation
import Testing

@testable import OtherworldsApp

@Suite("NarrativeEndpoint — correct paths and bodies via mock")
struct NarrativeEndpointTests {

    private func makeEndpoint() -> (NarrativeEndpoint, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = NarrativeEndpoint(client: mock)
        return (endpoint, mock)
    }

    @Test func listSessions_callsCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = "[]".data(using: .utf8)!
        mock.nextResult = .success(json)

        let _: [NarrativeSessionSummary] = try await endpoint.listSessions()

        #expect(mock.calls.count == 1)
        #expect(mock.calls[0].method == "GET")
        #expect(mock.calls[0].path == "/api/v1/narrative")
    }

    @Test func getSession_includesIdInPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let sessionId = UUID()
        let json = """
            {
                "session_id": "\(sessionId)",
                "current_beat_id": null,
                "choice_ids": [],
                "current_scene_id": null,
                "scene_history": [],
                "active_choice_options": [],
                "version": 0
            }
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let _ = try await endpoint.getSession(id: sessionId)

        #expect(mock.calls[0].path == "/api/v1/narrative/\(sessionId)")
        #expect(mock.calls[0].method == "GET")
    }

    @Test func advanceBeat_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":[]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = AdvanceBeatRequest(sessionId: UUID())
        let _ = try await endpoint.advanceBeat(request: request)

        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/narrative/advance-beat")
    }

    @Test func presentChoice_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":[]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = PresentChoiceRequest(sessionId: UUID())
        let _ = try await endpoint.presentChoice(request: request)

        #expect(mock.calls[0].path == "/api/v1/narrative/present-choice")
    }

    @Test func enterScene_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":[]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = EnterSceneRequest(
            sessionId: UUID(),
            sceneId: "intro",
            narrativeText: "Welcome.",
            choices: [],
            npcRefs: nil
        )
        let _ = try await endpoint.enterScene(request: request)

        #expect(mock.calls[0].path == "/api/v1/narrative/enter-scene")
    }

    @Test func selectChoice_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":[]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = SelectChoiceRequest(
            sessionId: UUID(),
            choiceIndex: 0,
            targetScene: TargetSceneRequest(
                sceneId: "cave",
                narrativeText: "Dark.",
                choices: [],
                npcRefs: nil
            )
        )
        let _ = try await endpoint.selectChoice(request: request)

        #expect(mock.calls[0].path == "/api/v1/narrative/select-choice")
    }

    @Test func archiveSession_deletesCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let sessionId = UUID()
        let json = """
            {"event_ids":[]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let _ = try await endpoint.archiveSession(id: sessionId)

        #expect(mock.calls[0].method == "DELETE")
        #expect(mock.calls[0].path == "/api/v1/narrative/\(sessionId)")
    }
}
