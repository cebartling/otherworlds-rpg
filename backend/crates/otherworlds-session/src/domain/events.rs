//! Domain events for the Session & Progress context.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Emitted when a campaign run is started.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignRunStarted {
    /// The campaign run identifier.
    pub run_id: Uuid,
    /// The campaign identifier.
    pub campaign_id: Uuid,
}

/// Emitted when a checkpoint is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointCreated {
    /// The campaign run identifier.
    pub run_id: Uuid,
    /// The checkpoint identifier.
    pub checkpoint_id: Uuid,
}

/// Emitted when a timeline is branched.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineBranched {
    /// The original campaign run identifier.
    pub source_run_id: Uuid,
    /// The new branch's campaign run identifier.
    pub branch_run_id: Uuid,
    /// The checkpoint to branch from.
    pub from_checkpoint_id: Uuid,
}
