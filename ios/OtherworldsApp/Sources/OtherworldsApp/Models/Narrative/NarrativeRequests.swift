import Foundation

/// Request body for POST /api/v1/narrative/advance-beat.
struct AdvanceBeatRequest: Codable, Equatable, Sendable {
    let sessionId: UUID
}

/// Request body for POST /api/v1/narrative/present-choice.
struct PresentChoiceRequest: Codable, Equatable, Sendable {
    let sessionId: UUID
}

/// A choice option within an EnterSceneRequest.
struct ChoiceOptionRequest: Codable, Equatable, Sendable {
    let label: String
    let targetSceneId: String
}

/// Request body for POST /api/v1/narrative/enter-scene.
struct EnterSceneRequest: Codable, Equatable, Sendable {
    let sessionId: UUID
    let sceneId: String
    let narrativeText: String
    let choices: [ChoiceOptionRequest]
    let npcRefs: [String]?
}

/// Nested target scene data for select-choice.
struct TargetSceneRequest: Codable, Equatable, Sendable {
    let sceneId: String
    let narrativeText: String
    let choices: [ChoiceOptionRequest]
    let npcRefs: [String]?
}

/// Request body for POST /api/v1/narrative/select-choice.
struct SelectChoiceRequest: Codable, Equatable, Sendable {
    let sessionId: UUID
    let choiceIndex: Int
    let targetScene: TargetSceneRequest
}
