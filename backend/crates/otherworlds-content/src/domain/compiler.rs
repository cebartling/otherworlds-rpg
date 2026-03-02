//! Content Authoring — campaign compiler.
//!
//! Converts a `ParsedCampaign` into a `CompiledCampaign` with
//! HashMap-indexed scenes and NPCs for O(1) runtime lookup.

use std::collections::HashMap;

use super::campaign_model::{
    CompiledCampaign, CompiledChoice, CompiledNpc, CompiledScene, ParsedCampaign,
};

/// Compiles a parsed campaign into an indexed runtime representation.
///
/// Converts vec-based parsed structures into HashMap-indexed compiled
/// structures. Should only be called after validation passes.
#[must_use]
pub fn compile_parsed_campaign(parsed: &ParsedCampaign) -> CompiledCampaign {
    let scenes: HashMap<String, CompiledScene> = parsed
        .scenes
        .iter()
        .map(|s| {
            let compiled_choices: Vec<CompiledChoice> = s
                .choices
                .iter()
                .map(|c| CompiledChoice {
                    label: c.label.clone(),
                    target_scene_id: c.target.clone(),
                })
                .collect();

            let scene = CompiledScene {
                id: s.id.clone(),
                narrative_text: s.narrative_text.clone(),
                choices: compiled_choices,
                npc_refs: s.npc_refs.clone(),
            };
            (s.id.clone(), scene)
        })
        .collect();

    let npcs: HashMap<String, CompiledNpc> = parsed
        .npcs
        .iter()
        .map(|n| {
            let npc = CompiledNpc {
                id: n.id.clone(),
                name: n.name.clone(),
                disposition: n.disposition.clone(),
            };
            (n.id.clone(), npc)
        })
        .collect();

    CompiledCampaign {
        title: parsed.front_matter.title.clone(),
        description: parsed.front_matter.description.clone(),
        min_engine_version: parsed.front_matter.min_engine_version,
        scenes,
        npcs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::campaign_model::{
        CampaignFrontMatter, ParsedChoice, ParsedNpc, ParsedScene,
    };

    #[test]
    fn test_compile_minimal_campaign() {
        let parsed = ParsedCampaign {
            front_matter: CampaignFrontMatter {
                title: "Test".to_owned(),
                description: None,
                min_engine_version: None,
            },
            scenes: vec![ParsedScene {
                id: "start".to_owned(),
                narrative_text: "Hello.".to_owned(),
                choices: Vec::new(),
                npc_refs: Vec::new(),
            }],
            npcs: Vec::new(),
        };

        let compiled = compile_parsed_campaign(&parsed);
        assert_eq!(compiled.title, "Test");
        assert_eq!(compiled.scenes.len(), 1);
        assert!(compiled.scenes.contains_key("start"));
        assert_eq!(compiled.scenes["start"].narrative_text, "Hello.");
        assert!(compiled.npcs.is_empty());
    }

    #[test]
    fn test_compile_campaign_with_npcs() {
        let parsed = ParsedCampaign {
            front_matter: CampaignFrontMatter {
                title: "Test".to_owned(),
                description: Some("A test campaign".to_owned()),
                min_engine_version: Some(1),
            },
            scenes: vec![ParsedScene {
                id: "start".to_owned(),
                narrative_text: "Hello.".to_owned(),
                choices: vec![ParsedChoice {
                    label: "Go".to_owned(),
                    target: "start".to_owned(),
                }],
                npc_refs: vec!["guard".to_owned()],
            }],
            npcs: vec![ParsedNpc {
                id: "guard".to_owned(),
                name: "Guard".to_owned(),
                disposition: Some("neutral".to_owned()),
            }],
        };

        let compiled = compile_parsed_campaign(&parsed);
        assert_eq!(compiled.description, Some("A test campaign".to_owned()));
        assert_eq!(compiled.min_engine_version, Some(1));
        assert_eq!(compiled.npcs.len(), 1);
        assert!(compiled.npcs.contains_key("guard"));
        assert_eq!(compiled.npcs["guard"].name, "Guard");
        assert_eq!(
            compiled.npcs["guard"].disposition,
            Some("neutral".to_owned())
        );
        assert_eq!(compiled.scenes["start"].choices.len(), 1);
        assert_eq!(compiled.scenes["start"].choices[0].target_scene_id, "start");
    }

    #[test]
    fn test_compiled_campaign_json_round_trip() {
        let parsed = ParsedCampaign {
            front_matter: CampaignFrontMatter {
                title: "Round Trip".to_owned(),
                description: None,
                min_engine_version: None,
            },
            scenes: vec![ParsedScene {
                id: "start".to_owned(),
                narrative_text: "Hello.".to_owned(),
                choices: Vec::new(),
                npc_refs: Vec::new(),
            }],
            npcs: Vec::new(),
        };

        let compiled = compile_parsed_campaign(&parsed);
        let json = serde_json::to_string(&compiled).unwrap();
        let deserialized: CompiledCampaign = serde_json::from_str(&json).unwrap();
        assert_eq!(compiled, deserialized);
    }
}
