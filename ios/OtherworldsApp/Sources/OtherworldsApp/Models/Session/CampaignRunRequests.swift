import Foundation

/// Request to start a new campaign run.
struct StartCampaignRunRequest: Codable, Equatable, Sendable {
    let campaignId: UUID?
}

/// Request to create a checkpoint in a campaign run.
struct CreateCheckpointRequest: Codable, Equatable, Sendable {
    let runId: UUID
    let checkpointId: UUID
}

/// Request to branch a timeline from a checkpoint.
struct BranchTimelineRequest: Codable, Equatable, Sendable {
    let runId: UUID
    let fromCheckpointId: UUID
    let newRunId: UUID
}
