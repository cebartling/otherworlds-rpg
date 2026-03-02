//! Content Authoring — campaign data model types for parsing, validation, and compilation.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Front-matter metadata extracted from the YAML block at the top of campaign source.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CampaignFrontMatter {
    /// Campaign title (required).
    pub title: String,
    /// Optional campaign description.
    pub description: Option<String>,
    /// Minimum engine version required to run this campaign.
    pub min_engine_version: Option<u32>,
}

/// A choice within a scene, linking to another scene by ID.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedChoice {
    /// Display label for the choice.
    pub label: String,
    /// Target scene ID this choice leads to.
    pub target: String,
}

/// A scene parsed from the campaign Markdown source.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedScene {
    /// Unique scene identifier (from `# Scene: <id>`).
    pub id: String,
    /// Narrative body text for the scene.
    pub narrative_text: String,
    /// Choices available in this scene.
    pub choices: Vec<ParsedChoice>,
    /// NPC IDs referenced in this scene.
    pub npc_refs: Vec<String>,
}

/// An NPC parsed from the campaign Markdown source.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedNpc {
    /// Unique NPC identifier (from `# NPC: <id>`).
    pub id: String,
    /// NPC display name.
    pub name: String,
    /// Optional NPC disposition.
    pub disposition: Option<String>,
}

/// Intermediate representation of a fully parsed campaign.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedCampaign {
    /// Campaign metadata from YAML front-matter.
    pub front_matter: CampaignFrontMatter,
    /// Scenes in document order.
    pub scenes: Vec<ParsedScene>,
    /// NPC definitions in document order.
    pub npcs: Vec<ParsedNpc>,
}

/// A compiled choice with resolved scene reference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompiledChoice {
    /// Display label for the choice.
    pub label: String,
    /// Target scene ID this choice leads to.
    pub target_scene_id: String,
}

/// A compiled scene indexed for O(1) lookup.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompiledScene {
    /// Unique scene identifier.
    pub id: String,
    /// Narrative body text for the scene.
    pub narrative_text: String,
    /// Choices available in this scene.
    pub choices: Vec<CompiledChoice>,
    /// NPC IDs referenced in this scene.
    pub npc_refs: Vec<String>,
}

/// A compiled NPC indexed for O(1) lookup.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompiledNpc {
    /// Unique NPC identifier.
    pub id: String,
    /// NPC display name.
    pub name: String,
    /// Optional NPC disposition.
    pub disposition: Option<String>,
}

/// Compiled campaign data optimised for runtime access.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompiledCampaign {
    /// Campaign title.
    pub title: String,
    /// Optional campaign description.
    pub description: Option<String>,
    /// Minimum engine version required.
    pub min_engine_version: Option<u32>,
    /// Scenes indexed by scene ID.
    pub scenes: HashMap<String, CompiledScene>,
    /// NPCs indexed by NPC ID.
    pub npcs: HashMap<String, CompiledNpc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_campaign_front_matter_json_round_trip() {
        let fm = CampaignFrontMatter {
            title: "The Lost Temple".to_owned(),
            description: Some("An adventure".to_owned()),
            min_engine_version: Some(1),
        };
        let json = serde_json::to_string(&fm).unwrap();
        let deserialized: CampaignFrontMatter = serde_json::from_str(&json).unwrap();
        assert_eq!(fm, deserialized);
    }

    #[test]
    fn test_parsed_campaign_json_round_trip() {
        let parsed = ParsedCampaign {
            front_matter: CampaignFrontMatter {
                title: "Test".to_owned(),
                description: None,
                min_engine_version: None,
            },
            scenes: vec![ParsedScene {
                id: "start".to_owned(),
                narrative_text: "Hello.".to_owned(),
                choices: vec![ParsedChoice {
                    label: "Go".to_owned(),
                    target: "end".to_owned(),
                }],
                npc_refs: vec!["guard".to_owned()],
            }],
            npcs: vec![ParsedNpc {
                id: "guard".to_owned(),
                name: "Guard".to_owned(),
                disposition: Some("neutral".to_owned()),
            }],
        };
        let json = serde_json::to_string(&parsed).unwrap();
        let deserialized: ParsedCampaign = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, deserialized);
    }

    #[test]
    fn test_compiled_campaign_json_round_trip() {
        let mut scenes = HashMap::new();
        scenes.insert(
            "start".to_owned(),
            CompiledScene {
                id: "start".to_owned(),
                narrative_text: "Hello.".to_owned(),
                choices: vec![CompiledChoice {
                    label: "Go".to_owned(),
                    target_scene_id: "end".to_owned(),
                }],
                npc_refs: vec!["guard".to_owned()],
            },
        );
        let mut npcs = HashMap::new();
        npcs.insert(
            "guard".to_owned(),
            CompiledNpc {
                id: "guard".to_owned(),
                name: "Guard".to_owned(),
                disposition: Some("neutral".to_owned()),
            },
        );
        let compiled = CompiledCampaign {
            title: "Test".to_owned(),
            description: None,
            min_engine_version: None,
            scenes,
            npcs,
        };
        let json = serde_json::to_string(&compiled).unwrap();
        let deserialized: CompiledCampaign = serde_json::from_str(&json).unwrap();
        assert_eq!(compiled, deserialized);
    }

    #[test]
    fn test_compiled_campaign_optional_fields_default_to_none() {
        let compiled = CompiledCampaign {
            title: "Minimal".to_owned(),
            description: None,
            min_engine_version: None,
            scenes: HashMap::new(),
            npcs: HashMap::new(),
        };
        assert!(compiled.description.is_none());
        assert!(compiled.min_engine_version.is_none());
    }
}
