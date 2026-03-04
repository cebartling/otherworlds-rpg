import Foundation

/// A single choice option within a narrative scene.
struct ChoiceOption: Codable, Equatable, Sendable {
    let label: String
    let targetSceneId: String
}
