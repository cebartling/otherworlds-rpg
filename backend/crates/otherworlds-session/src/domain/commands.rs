//! Commands for the Session & Progress context.

use uuid::Uuid;

/// Command to start a new campaign run.
#[derive(Debug, Clone)]
pub struct StartCampaignRun {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The campaign to run.
    pub campaign_id: Uuid,
}

/// Command to create a checkpoint.
#[derive(Debug, Clone)]
pub struct CreateCheckpoint {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The campaign run identifier.
    pub run_id: Uuid,
}

/// Command to branch a timeline.
#[derive(Debug, Clone)]
pub struct BranchTimeline {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The campaign run to branch from.
    pub source_run_id: Uuid,
    /// The checkpoint to branch from.
    pub from_checkpoint_id: Uuid,
}
