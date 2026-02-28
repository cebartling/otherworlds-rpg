//! Commands for the Content Authoring context.

use uuid::Uuid;

/// Command to ingest a campaign from source files.
#[derive(Debug, Clone)]
pub struct IngestCampaign {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The campaign source path or identifier.
    pub source: String,
}

/// Command to validate a campaign.
#[derive(Debug, Clone)]
pub struct ValidateCampaign {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The campaign identifier.
    pub campaign_id: Uuid,
}

/// Command to compile a campaign into runtime format.
#[derive(Debug, Clone)]
pub struct CompileCampaign {
    /// The correlation ID for tracing.
    pub correlation_id: Uuid,
    /// The campaign identifier.
    pub campaign_id: Uuid,
}
