//! Commands for the Session & Progress context.

use otherworlds_core::command::Command;
use uuid::Uuid;

/// Command to start a new campaign run.
#[derive(Debug, Clone)]
pub struct StartCampaignRun {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The campaign to run.
    pub campaign_id: Uuid,
}

impl Command for StartCampaignRun {
    fn command_type(&self) -> &'static str {
        "session.start_campaign_run"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}

/// Command to create a checkpoint.
#[derive(Debug, Clone)]
pub struct CreateCheckpoint {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The campaign run identifier.
    pub run_id: Uuid,
}

impl Command for CreateCheckpoint {
    fn command_type(&self) -> &'static str {
        "session.create_checkpoint"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
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

impl Command for BranchTimeline {
    fn command_type(&self) -> &'static str {
        "session.branch_timeline"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}

/// Command to register an aggregate from another bounded context with a campaign run.
#[derive(Debug, Clone)]
pub struct RegisterAggregate {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The campaign run to register with.
    pub run_id: Uuid,
    /// The bounded context name (e.g. "narrative", "character").
    pub context_name: String,
    /// The aggregate ID in the other context.
    pub aggregate_id: Uuid,
}

impl Command for RegisterAggregate {
    fn command_type(&self) -> &'static str {
        "session.register_aggregate"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}

/// Command to archive (soft-delete) a campaign run.
#[derive(Debug, Clone)]
pub struct ArchiveCampaignRun {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The campaign run to archive.
    pub run_id: Uuid,
}

impl Command for ArchiveCampaignRun {
    fn command_type(&self) -> &'static str {
        "session.archive_campaign_run"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}
