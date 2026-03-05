import Foundation

/// Response returned after a command is successfully handled by the backend.
///
/// Contains the IDs of domain events produced and persisted.
struct CommandResponse: Codable, Equatable, Sendable {
    let eventIds: [UUID]
    let aggregateId: UUID?
}
