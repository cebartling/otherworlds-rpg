import Foundation
import Testing

@testable import OtherworldsApp

@Suite("WorldSnapshot Models — Codable round-trips")
struct WorldSnapshotModelsTests {

    @Test func worldSnapshotSummary_decodesFromSnakeCaseJSON() throws {
        let worldId = UUID()
        let json = """
            {"world_id":"\(worldId)","fact_count":3,"flag_count":2,"version":1}
            """.data(using: .utf8)!

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let decoded = try decoder.decode(WorldSnapshotSummary.self, from: json)

        #expect(decoded.worldId == worldId)
        #expect(decoded.factCount == 3)
        #expect(decoded.flagCount == 2)
        #expect(decoded.version == 1)
        #expect(decoded.id == worldId)
    }

    @Test func worldSnapshotDetail_decodesComplexTypes() throws {
        let worldId = UUID()
        let entityId = UUID()
        let json = """
            {"world_id":"\(worldId)","facts":["The gate is open"],"flags":{"quest_started":true},"disposition_entity_ids":["\(entityId)"],"version":2}
            """.data(using: .utf8)!

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let decoded = try decoder.decode(WorldSnapshotDetail.self, from: json)

        #expect(decoded.facts == ["The gate is open"])
        #expect(decoded.flags == ["quest_started": true])
        #expect(decoded.dispositionEntityIds == [entityId])
    }

    @Test func applyEffectRequest_encodesToSnakeCase() throws {
        let worldId = UUID()
        let request = ApplyEffectRequest(worldId: worldId, factKey: "gate_opened")
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(request)
        let json = String(data: data, encoding: .utf8)!

        #expect(json.contains("\"fact_key\":\"gate_opened\""))
        #expect(json.contains("\"world_id\""))
    }

    @Test func setFlagRequest_encodesToSnakeCase() throws {
        let worldId = UUID()
        let request = SetFlagRequest(worldId: worldId, flagKey: "quest_active", value: true)
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(request)
        let json = String(data: data, encoding: .utf8)!

        #expect(json.contains("\"flag_key\":\"quest_active\""))
        #expect(json.contains("\"value\":true"))
    }

    @Test func updateDispositionRequest_encodesToSnakeCase() throws {
        let worldId = UUID()
        let entityId = UUID()
        let request = UpdateDispositionRequest(worldId: worldId, entityId: entityId)
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(request)
        let json = String(data: data, encoding: .utf8)!

        #expect(json.contains("\"entity_id\""))
        #expect(json.contains("\"world_id\""))
    }
}
