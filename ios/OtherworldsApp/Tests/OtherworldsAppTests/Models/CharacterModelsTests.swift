import Foundation
import Testing

@testable import OtherworldsApp

@Suite("Character Models — Codable round-trips")
struct CharacterModelsTests {

    // MARK: - CreateCharacterRequest

    @Test func createCharacterRequest_encodesToSnakeCase() throws {
        let request = CreateCharacterRequest(name: "Alaric")
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(request)
        let json = String(data: data, encoding: .utf8)!

        #expect(json.contains("\"name\":\"Alaric\""))
    }

    // MARK: - ModifyAttributeRequest

    @Test func modifyAttributeRequest_encodesToSnakeCase() throws {
        let characterId = UUID()
        let request = ModifyAttributeRequest(
            characterId: characterId,
            attribute: "strength",
            newValue: 18
        )
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(request)
        let json = String(data: data, encoding: .utf8)!

        #expect(json.contains("\"character_id\""))
        #expect(json.contains("\"new_value\":18"))
        #expect(json.contains("\"attribute\":\"strength\""))
    }

    // MARK: - AwardExperienceRequest

    @Test func awardExperienceRequest_encodesToSnakeCase() throws {
        let characterId = UUID()
        let request = AwardExperienceRequest(characterId: characterId, amount: 100)
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        let data = try encoder.encode(request)
        let json = String(data: data, encoding: .utf8)!

        #expect(json.contains("\"character_id\""))
        #expect(json.contains("\"amount\":100"))
    }

    // MARK: - CharacterSummary

    @Test func characterSummary_decodesFromSnakeCaseJSON() throws {
        let characterId = UUID()
        let json = """
            {"character_id":"\(characterId)","name":"Alaric","experience":250,"version":3}
            """.data(using: .utf8)!

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let decoded = try decoder.decode(CharacterSummary.self, from: json)

        #expect(decoded.characterId == characterId)
        #expect(decoded.name == "Alaric")
        #expect(decoded.experience == 250)
        #expect(decoded.version == 3)
        #expect(decoded.id == characterId)
    }

    @Test func characterSummary_decodesWithNilName() throws {
        let characterId = UUID()
        let json = """
            {"character_id":"\(characterId)","name":null,"experience":0,"version":1}
            """.data(using: .utf8)!

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let decoded = try decoder.decode(CharacterSummary.self, from: json)

        #expect(decoded.characterId == characterId)
        #expect(decoded.name == nil)
        #expect(decoded.experience == 0)
    }

    // MARK: - CharacterDetail

    @Test func characterDetail_decodesFromSnakeCaseJSON() throws {
        let characterId = UUID()
        let json = """
            {
                "character_id":"\(characterId)",
                "name":"Alaric",
                "attributes":{"strength":18,"dexterity":14},
                "experience":250,
                "version":3
            }
            """.data(using: .utf8)!

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let decoded = try decoder.decode(CharacterDetail.self, from: json)

        #expect(decoded.characterId == characterId)
        #expect(decoded.name == "Alaric")
        #expect(decoded.attributes["strength"] == 18)
        #expect(decoded.attributes["dexterity"] == 14)
        #expect(decoded.experience == 250)
        #expect(decoded.version == 3)
        #expect(decoded.id == characterId)
    }

    @Test func characterDetail_decodesWithNilNameAndEmptyAttributes() throws {
        let characterId = UUID()
        let json = """
            {
                "character_id":"\(characterId)",
                "name":null,
                "attributes":{},
                "experience":0,
                "version":1
            }
            """.data(using: .utf8)!

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let decoded = try decoder.decode(CharacterDetail.self, from: json)

        #expect(decoded.name == nil)
        #expect(decoded.attributes.isEmpty)
    }
}
