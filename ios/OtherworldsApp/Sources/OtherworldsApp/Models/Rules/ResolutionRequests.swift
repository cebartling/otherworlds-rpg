import Foundation

/// A single effect entry for the produce-effects command.
struct EffectEntry: Codable, Equatable, Sendable {
    let targetId: UUID
    let effectType: String
    let magnitude: Int32
}

/// Request to declare intent for a resolution.
struct DeclareIntentRequest: Codable, Equatable, Sendable {
    let resolutionId: UUID
    let intentId: UUID
    let actionType: String
    let skill: String?
    let targetId: UUID?
    let difficultyClass: Int32
    let modifier: Int32
}

/// Request to resolve a check.
struct ResolveCheckRequest: Codable, Equatable, Sendable {
    let resolutionId: UUID
}

/// Request to produce effects from a resolution.
struct ProduceEffectsRequest: Codable, Equatable, Sendable {
    let resolutionId: UUID
    let effects: [EffectEntry]
}
