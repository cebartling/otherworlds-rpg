import Foundation

/// A single effect to apply as part of a resolved action.
struct EffectSpec: Codable, Equatable, Sendable {
    let effectType: String
    let targetId: UUID?
    let payload: [String: String]
}

/// Request to orchestrate the full play loop: rules intent → check → effects → world state → narrative.
struct ResolveActionRequest: Codable, Equatable, Sendable {
    let sessionId: UUID
    let worldId: UUID
    let actionType: String
    let skill: String?
    let targetId: UUID?
    let difficultyClass: Int32
    let modifier: Int32
    let effects: [EffectSpec]
}

/// Response from the orchestrated play loop, containing event IDs from each phase.
struct ResolveActionResponse: Codable, Equatable, Sendable {
    let correlationId: UUID
    let resolutionId: UUID
    let intentEventIds: [UUID]
    let checkEventIds: [UUID]
    let effectsEventIds: [UUID]
    let worldStateEventIds: [UUID]
    let narrativeEventIds: [UUID]
}
