import Foundation

/// Detailed view of a single character.
///
/// Corresponds to GET /api/v1/characters/:id response.
struct CharacterDetail: Codable, Equatable, Identifiable, Sendable {
    let characterId: UUID
    let name: String?
    let attributes: [String: Int32]
    let experience: UInt32
    let version: Int

    var id: UUID { characterId }
}
