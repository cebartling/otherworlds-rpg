//! Domain events for the Content Authoring context.

use otherworlds_core::event::{DomainEvent, EventMetadata};
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

/// Event payload variants for the Content Authoring context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentEventKind {
    /// A campaign has been ingested from source files.
    CampaignIngested(CampaignIngested),
    /// A campaign has passed validation.
    CampaignValidated(CampaignValidated),
    /// A campaign has been compiled into runtime format.
    CampaignCompiled(CampaignCompiled),
}

/// Domain event envelope for the Content Authoring context.
#[derive(Debug, Clone)]
pub struct ContentEvent {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// Event-specific payload.
    pub kind: ContentEventKind,
}

impl DomainEvent for ContentEvent {
    fn event_type(&self) -> &'static str {
        match &self.kind {
            ContentEventKind::CampaignIngested(_) => "content.campaign_ingested",
            ContentEventKind::CampaignValidated(_) => "content.campaign_validated",
            ContentEventKind::CampaignCompiled(_) => "content.campaign_compiled",
        }
    }

    fn to_payload(&self) -> serde_json::Value {
        // Serialization of derived Serialize types to Value is infallible.
        serde_json::to_value(&self.kind).expect("ContentEventKind serialization is infallible")
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
