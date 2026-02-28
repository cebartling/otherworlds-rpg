//! Commands for the Content Authoring context.

use otherworlds_core::command::Command;
use uuid::Uuid;

/// Command to ingest a campaign from source files.
#[derive(Debug, Clone)]
pub struct IngestCampaign {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The campaign source path or identifier.
    pub source: String,
}

impl Command for IngestCampaign {
    fn command_type(&self) -> &'static str {
        "content.ingest_campaign"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}

/// Command to validate a campaign.
#[derive(Debug, Clone)]
pub struct ValidateCampaign {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The campaign identifier.
    pub campaign_id: Uuid,
}

impl Command for ValidateCampaign {
    fn command_type(&self) -> &'static str {
        "content.validate_campaign"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}

/// Command to compile a campaign into runtime format.
#[derive(Debug, Clone)]
pub struct CompileCampaign {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The campaign identifier.
    pub campaign_id: Uuid,
}

impl Command for CompileCampaign {
    fn command_type(&self) -> &'static str {
        "content.compile_campaign"
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
}
