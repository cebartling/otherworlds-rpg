import Foundation
import Testing

@testable import OtherworldsApp

@Suite("Narrative Models — Codable round-trips with real JSON shapes")
struct NarrativeModelsTests {

    private var decoder: JSONDecoder {
        let d = JSONDecoder()
        d.keyDecodingStrategy = .convertFromSnakeCase
        return d
    }

    private var encoder: JSONEncoder {
        let e = JSONEncoder()
        e.keyEncodingStrategy = .convertToSnakeCase
        return e
    }

    // MARK: - ChoiceOption

    @Test func choiceOption_decodesFromBackendJSON() throws {
        let json = """
            {"label":"Go north","target_scene_id":"forest-clearing"}
            """.data(using: .utf8)!

        let decoded = try decoder.decode(ChoiceOption.self, from: json)

        #expect(decoded.label == "Go north")
        #expect(decoded.targetSceneId == "forest-clearing")
    }

    // MARK: - NarrativeSessionSummary

    @Test func sessionSummary_decodesFromBackendJSON() throws {
        let sessionId = UUID()
        let beatId = UUID()
        let json = """
            {
                "session_id": "\(sessionId)",
                "current_beat_id": "\(beatId)",
                "current_scene_id": "tavern",
                "version": 5
            }
            """.data(using: .utf8)!

        let decoded = try decoder.decode(NarrativeSessionSummary.self, from: json)

        #expect(decoded.sessionId == sessionId)
        #expect(decoded.currentBeatId == beatId)
        #expect(decoded.currentSceneId == "tavern")
        #expect(decoded.version == 5)
        #expect(decoded.id == sessionId)
    }

    @Test func sessionSummary_decodesNullOptionals() throws {
        let sessionId = UUID()
        let json = """
            {
                "session_id": "\(sessionId)",
                "current_beat_id": null,
                "current_scene_id": null,
                "version": 0
            }
            """.data(using: .utf8)!

        let decoded = try decoder.decode(NarrativeSessionSummary.self, from: json)

        #expect(decoded.currentBeatId == nil)
        #expect(decoded.currentSceneId == nil)
    }

    // MARK: - NarrativeSessionView

    @Test func sessionView_decodesFullResponse() throws {
        let sessionId = UUID()
        let beatId = UUID()
        let choiceId = UUID()
        let json = """
            {
                "session_id": "\(sessionId)",
                "current_beat_id": "\(beatId)",
                "choice_ids": ["\(choiceId)"],
                "current_scene_id": "tavern",
                "scene_history": ["intro", "tavern"],
                "active_choice_options": [
                    {"label": "Talk to barkeeper", "target_scene_id": "barkeeper-chat"}
                ],
                "version": 3
            }
            """.data(using: .utf8)!

        let decoded = try decoder.decode(NarrativeSessionView.self, from: json)

        #expect(decoded.sessionId == sessionId)
        #expect(decoded.currentBeatId == beatId)
        #expect(decoded.choiceIds == [choiceId])
        #expect(decoded.currentSceneId == "tavern")
        #expect(decoded.sceneHistory == ["intro", "tavern"])
        #expect(decoded.activeChoiceOptions.count == 1)
        #expect(decoded.activeChoiceOptions[0].label == "Talk to barkeeper")
        #expect(decoded.version == 3)
    }

    @Test func sessionView_decodesEmptyArrays() throws {
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

        let decoded = try decoder.decode(NarrativeSessionView.self, from: json)

        #expect(decoded.choiceIds.isEmpty)
        #expect(decoded.sceneHistory.isEmpty)
        #expect(decoded.activeChoiceOptions.isEmpty)
    }

    // MARK: - Request encoding

    @Test func advanceBeatRequest_encodesToSnakeCase() throws {
        let sessionId = UUID()
        let request = AdvanceBeatRequest(sessionId: sessionId)
        let data = try encoder.encode(request)
        let dict = try JSONSerialization.jsonObject(with: data) as! [String: Any]

        #expect(dict["session_id"] as? String == sessionId.uuidString)
    }

    @Test func enterSceneRequest_encodesFullStructure() throws {
        let sessionId = UUID()
        let request = EnterSceneRequest(
            sessionId: sessionId,
            sceneId: "forest",
            narrativeText: "You enter a dark forest.",
            choices: [
                ChoiceOptionRequest(label: "Go deeper", targetSceneId: "deep-forest"),
            ],
            npcRefs: ["elf-ranger"]
        )
        let data = try encoder.encode(request)
        let dict = try JSONSerialization.jsonObject(with: data) as! [String: Any]

        #expect(dict["session_id"] as? String == sessionId.uuidString)
        #expect(dict["scene_id"] as? String == "forest")
        #expect(dict["narrative_text"] as? String == "You enter a dark forest.")
        #expect((dict["choices"] as? [[String: Any]])?.count == 1)
        #expect((dict["npc_refs"] as? [String]) == ["elf-ranger"])
    }

    @Test func selectChoiceRequest_encodesNestedTargetScene() throws {
        let sessionId = UUID()
        let request = SelectChoiceRequest(
            sessionId: sessionId,
            choiceIndex: 1,
            targetScene: TargetSceneRequest(
                sceneId: "cave",
                narrativeText: "A damp cave.",
                choices: [],
                npcRefs: nil
            )
        )
        let data = try encoder.encode(request)
        let dict = try JSONSerialization.jsonObject(with: data) as! [String: Any]

        #expect(dict["choice_index"] as? Int == 1)
        let target = dict["target_scene"] as? [String: Any]
        #expect(target?["scene_id"] as? String == "cave")
    }
}
