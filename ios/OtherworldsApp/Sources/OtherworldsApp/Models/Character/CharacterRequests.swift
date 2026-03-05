import Foundation

/// Request to create a new character.
struct CreateCharacterRequest: Codable, Equatable, Sendable {
    let name: String
}

/// Request to modify a character attribute value.
struct ModifyAttributeRequest: Codable, Equatable, Sendable {
    let characterId: UUID
    let attribute: String
    let newValue: Int32
}

/// Request to award experience points to a character.
struct AwardExperienceRequest: Codable, Equatable, Sendable {
    let characterId: UUID
    let amount: UInt32
}
