import Foundation
import Testing

@testable import OtherworldsApp

@Suite("Resolution Models — Codable round-trips")
struct ResolutionModelsTests {

    @Test func resolutionSummary_decodesFromSnakeCaseJSON() throws {
        let resolutionId = UUID()
        let json = """
            {"resolution_id":"\(resolutionId)","phase":"Created","outcome":null,"version":1}
            """.data(using: .utf8)!

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let decoded = try decoder.decode(ResolutionSummary.self, from: json)

        #expect(decoded.resolutionId == resolutionId)
        #expect(decoded.phase == "Created")
        #expect(decoded.outcome == nil)
        #expect(decoded.version == 1)
        #expect(decoded.id == resolutionId)
    }

    @Test func resolutionSummary_decodesWithOutcome() throws {
        let resolutionId = UUID()
        let json = """
            {"resolution_id":"\(resolutionId)","phase":"CheckResolved","outcome":"Success","version":2}
            """.data(using: .utf8)!

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let decoded = try decoder.decode(ResolutionSummary.self, from: json)

        #expect(decoded.outcome == "Success")
    }

    @Test func resolutionDetail_decodesNestedTypes() throws {
        let resolutionId = UUID()
        let intentId = UUID()
        let targetId = UUID()
        let json = """
            {
                "resolution_id":"\(resolutionId)",
                "phase":"EffectsProduced",
                "intent":{
                    "intent_id":"\(intentId)",
                    "action_type":"attack",
                    "skill":"swordsmanship",
                    "target_id":"\(targetId)",
                    "difficulty_class":15,
                    "modifier":3
                },
                "check_result":{
                    "roll":17,
                    "total":20,
                    "outcome":"Success"
                },
                "effects":[{
                    "target_id":"\(targetId)",
                    "effect_type":"damage",
                    "magnitude":8
                }],
                "version":4
            }
            """.data(using: .utf8)!

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let decoded = try decoder.decode(ResolutionDetail.self, from: json)

        #expect(decoded.intent?.actionType == "attack")
        #expect(decoded.intent?.skill == "swordsmanship")
        #expect(decoded.intent?.difficultyClass == 15)
        #expect(decoded.intent?.modifier == 3)
        #expect(decoded.checkResult?.roll == 17)
        #expect(decoded.checkResult?.total == 20)
        #expect(decoded.checkResult?.outcome == "Success")
        #expect(decoded.effects.count == 1)
        #expect(decoded.effects[0].effectType == "damage")
        #expect(decoded.effects[0].magnitude == 8)
    }

    @Test func declareIntentRequest_encodesToSnakeCase() throws {
        let request = DeclareIntentRequest(
            resolutionId: UUID(),
            intentId: UUID(),
            actionType: "attack",
            skill: "swordsmanship",
            targetId: UUID(),
            difficultyClass: 15,
            modifier: 3
        )
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(request)
        let json = String(data: data, encoding: .utf8)!

        #expect(json.contains("\"action_type\":\"attack\""))
        #expect(json.contains("\"difficulty_class\":15"))
        #expect(json.contains("\"modifier\":3"))
    }

    @Test func resolveCheckRequest_encodesToSnakeCase() throws {
        let resolutionId = UUID()
        let request = ResolveCheckRequest(resolutionId: resolutionId)
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(request)
        let json = String(data: data, encoding: .utf8)!

        #expect(json.contains("\"resolution_id\""))
    }

    @Test func produceEffectsRequest_encodesToSnakeCase() throws {
        let targetId = UUID()
        let request = ProduceEffectsRequest(
            resolutionId: UUID(),
            effects: [EffectEntry(targetId: targetId, effectType: "damage", magnitude: 10)]
        )
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(request)
        let json = String(data: data, encoding: .utf8)!

        #expect(json.contains("\"effect_type\":\"damage\""))
        #expect(json.contains("\"magnitude\":10"))
    }

    @Test func effectEntry_encodesToSnakeCase() throws {
        let entry = EffectEntry(targetId: UUID(), effectType: "heal", magnitude: 5)
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(entry)
        let json = String(data: data, encoding: .utf8)!

        #expect(json.contains("\"effect_type\":\"heal\""))
        #expect(json.contains("\"target_id\""))
    }
}
