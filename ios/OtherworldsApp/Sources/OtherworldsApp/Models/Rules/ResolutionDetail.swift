import Foundation

/// The declared intent for a resolution check.
struct DeclaredIntent: Codable, Equatable, Sendable {
    let intentId: UUID
    let actionType: String
    let skill: String?
    let targetId: UUID?
    let difficultyClass: Int32
    let modifier: Int32
}

/// The result of a resolution check.
struct CheckResult: Codable, Equatable, Sendable {
    let roll: Int32
    let total: Int32
    let outcome: String
}

/// A resolved effect produced by a resolution.
struct ResolvedEffect: Codable, Equatable, Sendable {
    let targetId: UUID
    let effectType: String
    let magnitude: Int32
}

/// Detailed view of a single resolution.
///
/// Corresponds to GET /api/v1/rules/:id response.
struct ResolutionDetail: Codable, Equatable, Identifiable, Sendable {
    let resolutionId: UUID
    let phase: String
    let intent: DeclaredIntent?
    let checkResult: CheckResult?
    let effects: [ResolvedEffect]
    let version: Int

    var id: UUID { resolutionId }
}
