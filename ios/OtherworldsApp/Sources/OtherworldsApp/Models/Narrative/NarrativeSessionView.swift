import Foundation

/// Full read-only view of a narrative session.
///
/// Corresponds to GET /api/v1/narrative/:id response.
struct NarrativeSessionView: Codable, Equatable, Sendable {
    let sessionId: UUID
    let currentBeatId: UUID?
    let choiceIds: [UUID]
    let currentSceneId: String?
    let sceneHistory: [String]
    let activeChoiceOptions: [ChoiceOption]
    let version: Int
}
