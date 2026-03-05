import Foundation

/// API client for the Character bounded context.
///
/// Routes are nested under /api/v1/characters on the backend.
struct CharacterEndpoint: Sendable {
    private static let base = "/api/v1/characters"

    private let client: HTTPClientProtocol

    init(client: HTTPClientProtocol) {
        self.client = client
    }

    /// GET /api/v1/characters — list all characters.
    func listCharacters() async throws -> [CharacterSummary] {
        try await client.get(path: Self.base, correlationId: nil)
    }

    /// GET /api/v1/characters/:id — get character detail.
    func getCharacter(id: UUID) async throws -> CharacterDetail {
        try await client.get(path: "\(Self.base)/\(id)", correlationId: nil)
    }

    /// POST /api/v1/characters/create
    func createCharacter(request: CreateCharacterRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/create", body: request, correlationId: nil)
    }

    /// POST /api/v1/characters/modify-attribute
    func modifyAttribute(request: ModifyAttributeRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/modify-attribute", body: request, correlationId: nil)
    }

    /// POST /api/v1/characters/award-experience
    func awardExperience(request: AwardExperienceRequest) async throws -> CommandResponse {
        try await client.post(path: "\(Self.base)/award-experience", body: request, correlationId: nil)
    }

    /// DELETE /api/v1/characters/:id — archive a character.
    func archiveCharacter(id: UUID) async throws -> CommandResponse {
        try await client.delete(path: "\(Self.base)/\(id)", correlationId: nil)
    }
}
