//! Domain events for the Content Authoring context.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Emitted when a campaign is ingested from source files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignIngested {
    /// The campaign identifier.
    pub campaign_id: Uuid,
    /// The campaign version hash.
    pub version_hash: String,
}

/// Emitted when a campaign passes validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignValidated {
    /// The campaign identifier.
    pub campaign_id: Uuid,
}

/// Emitted when a campaign is compiled into runtime format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignCompiled {
    /// The campaign identifier.
    pub campaign_id: Uuid,
    /// The compiled campaign version hash.
    pub version_hash: String,
}
