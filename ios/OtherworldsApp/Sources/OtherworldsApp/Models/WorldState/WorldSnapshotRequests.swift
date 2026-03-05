import Foundation

/// Request to apply an effect (add a fact) to a world snapshot.
struct ApplyEffectRequest: Codable, Equatable, Sendable {
    let worldId: UUID
    let factKey: String
}

/// Request to set a flag on a world snapshot.
struct SetFlagRequest: Codable, Equatable, Sendable {
    let worldId: UUID
    let flagKey: String
    let value: Bool
}

/// Request to update a disposition in a world snapshot.
struct UpdateDispositionRequest: Codable, Equatable, Sendable {
    let worldId: UUID
    let entityId: UUID
}
