import Foundation
import Testing

@testable import OtherworldsApp

@Suite("CharacterEndpoint — correct paths and bodies via mock")
struct CharacterEndpointTests {

    private func makeEndpoint() -> (CharacterEndpoint, MockHTTPClient) {
        let mock = MockHTTPClient()
        let endpoint = CharacterEndpoint(client: mock)
        return (endpoint, mock)
    }

    @Test func listCharacters_callsCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = "[]".data(using: .utf8)!
        mock.nextResult = .success(json)

        let _: [CharacterSummary] = try await endpoint.listCharacters()

        #expect(mock.calls.count == 1)
        #expect(mock.calls[0].method == "GET")
        #expect(mock.calls[0].path == "/api/v1/characters")
    }

    @Test func getCharacter_includesIdInPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let characterId = UUID()
        let json = """
            {
                "character_id": "\(characterId)",
                "name": "Alaric",
                "attributes": {},
                "experience": 0,
                "version": 1
            }
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let _ = try await endpoint.getCharacter(id: characterId)

        #expect(mock.calls[0].path == "/api/v1/characters/\(characterId)")
        #expect(mock.calls[0].method == "GET")
    }

    @Test func createCharacter_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":[]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = CreateCharacterRequest(name: "Alaric")
        let _ = try await endpoint.createCharacter(request: request)

        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/characters/create")
    }

    @Test func modifyAttribute_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":[]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = ModifyAttributeRequest(
            characterId: UUID(),
            attribute: "strength",
            newValue: 18
        )
        let _ = try await endpoint.modifyAttribute(request: request)

        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/characters/modify-attribute")
    }

    @Test func awardExperience_postsToCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let json = """
            {"event_ids":[]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let request = AwardExperienceRequest(characterId: UUID(), amount: 100)
        let _ = try await endpoint.awardExperience(request: request)

        #expect(mock.calls[0].method == "POST")
        #expect(mock.calls[0].path == "/api/v1/characters/award-experience")
    }

    @Test func archiveCharacter_deletesCorrectPath() async throws {
        let (endpoint, mock) = makeEndpoint()
        let characterId = UUID()
        let json = """
            {"event_ids":[]}
            """.data(using: .utf8)!
        mock.nextResult = .success(json)

        let _ = try await endpoint.archiveCharacter(id: characterId)

        #expect(mock.calls[0].method == "DELETE")
        #expect(mock.calls[0].path == "/api/v1/characters/\(characterId)")
    }
}
