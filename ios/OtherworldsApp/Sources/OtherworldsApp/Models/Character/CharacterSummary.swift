import Foundation

/// Summary view for listing characters.
///
/// Corresponds to GET /api/v1/characters response items.
struct CharacterSummary: Codable, Equatable, Identifiable, Sendable {
    let characterId: UUID
    let name: String?
    let experience: UInt32
    let version: Int

    var id: UUID { characterId }
}
