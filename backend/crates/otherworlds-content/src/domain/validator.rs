//! Content Authoring — campaign structural validation.
//!
//! Validates a `ParsedCampaign` against structural integrity rules
//! before compilation.

use std::collections::HashSet;

use otherworlds_core::error::DomainError;

use super::campaign_model::ParsedCampaign;

/// Validates a parsed campaign for structural correctness.
///
/// Checks seven rules:
/// 1. Front-matter title must be non-empty (after trim)
/// 2. At least one scene defined
/// 3. No duplicate scene IDs
/// 4. No duplicate NPC IDs
/// 5. All choice targets reference defined scene IDs
/// 6. All NPC refs in scenes reference defined NPC IDs
/// 7. Every NPC must have a non-empty name
///
/// # Errors
///
/// Returns `DomainError::Validation` with all errors joined by `"; "`.
pub fn validate_parsed_campaign(parsed: &ParsedCampaign) -> Result<(), DomainError> {
    let mut errors: Vec<String> = Vec::new();

    // Rule 1: Non-empty title.
    if parsed.front_matter.title.trim().is_empty() {
        errors.push("front-matter title must not be empty".to_owned());
    }

    // Rule 2: At least one scene.
    if parsed.scenes.is_empty() {
        errors.push("campaign must define at least one scene".to_owned());
    }

    // Rule 3: No duplicate scene IDs.
    let mut scene_ids = HashSet::new();
    for scene in &parsed.scenes {
        if !scene_ids.insert(&scene.id) {
            errors.push(format!("duplicate scene ID: {}", scene.id));
        }
    }

    // Rule 4: No duplicate NPC IDs.
    let mut npc_ids = HashSet::new();
    for npc in &parsed.npcs {
        if !npc_ids.insert(&npc.id) {
            errors.push(format!("duplicate NPC ID: {}", npc.id));
        }
    }

    // Rule 5: All choice targets reference defined scene IDs.
    for scene in &parsed.scenes {
        for choice in &scene.choices {
            if !scene_ids.contains(&choice.target) {
                errors.push(format!(
                    "scene '{}' has choice targeting undefined scene '{}'",
                    scene.id, choice.target
                ));
            }
        }
    }

    // Rule 6: All NPC refs in scenes reference defined NPC IDs.
    for scene in &parsed.scenes {
        for npc_ref in &scene.npc_refs {
            if !npc_ids.contains(npc_ref) {
                errors.push(format!(
                    "scene '{}' references undefined NPC '{npc_ref}'",
                    scene.id
                ));
            }
        }
    }

    // Rule 7: Every NPC must have a non-empty name.
    for npc in &parsed.npcs {
        if npc.name.trim().is_empty() {
            errors.push(format!("NPC '{}' must have a non-empty name", npc.id));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(DomainError::Validation(errors.join("; ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::campaign_model::{
        CampaignFrontMatter, ParsedChoice, ParsedNpc, ParsedScene,
    };

    fn valid_campaign() -> ParsedCampaign {
        ParsedCampaign {
            front_matter: CampaignFrontMatter {
                title: "Test Campaign".to_owned(),
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
        }
    }

    #[test]
    fn test_valid_campaign_passes() {
        let parsed = valid_campaign();
        assert!(validate_parsed_campaign(&parsed).is_ok());
    }

    #[test]
    fn test_empty_title_fails() {
        let mut parsed = valid_campaign();
        parsed.front_matter.title = "  ".to_owned();
        let err = validate_parsed_campaign(&parsed).unwrap_err();
        match err {
            DomainError::Validation(msg) => assert!(msg.contains("title must not be empty")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_no_scenes_fails() {
        let mut parsed = valid_campaign();
        parsed.scenes.clear();
        let err = validate_parsed_campaign(&parsed).unwrap_err();
        match err {
            DomainError::Validation(msg) => assert!(msg.contains("at least one scene")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_duplicate_scene_ids_fails() {
        let mut parsed = valid_campaign();
        parsed.scenes.push(ParsedScene {
            id: "start".to_owned(),
            narrative_text: "Duplicate.".to_owned(),
            choices: Vec::new(),
            npc_refs: Vec::new(),
        });
        let err = validate_parsed_campaign(&parsed).unwrap_err();
        match err {
            DomainError::Validation(msg) => assert!(msg.contains("duplicate scene ID: start")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_duplicate_npc_ids_fails() {
        let mut parsed = valid_campaign();
        parsed.npcs.push(ParsedNpc {
            id: "guard".to_owned(),
            name: "Guard A".to_owned(),
            disposition: None,
        });
        parsed.npcs.push(ParsedNpc {
            id: "guard".to_owned(),
            name: "Guard B".to_owned(),
            disposition: None,
        });
        let err = validate_parsed_campaign(&parsed).unwrap_err();
        match err {
            DomainError::Validation(msg) => assert!(msg.contains("duplicate NPC ID: guard")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_unresolved_choice_target_fails() {
        let mut parsed = valid_campaign();
        parsed.scenes[0].choices.push(ParsedChoice {
            label: "Go".to_owned(),
            target: "nonexistent".to_owned(),
        });
        let err = validate_parsed_campaign(&parsed).unwrap_err();
        match err {
            DomainError::Validation(msg) => {
                assert!(msg.contains("targeting undefined scene 'nonexistent'"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_unresolved_npc_ref_fails() {
        let mut parsed = valid_campaign();
        parsed.scenes[0].npc_refs.push("missing_npc".to_owned());
        let err = validate_parsed_campaign(&parsed).unwrap_err();
        match err {
            DomainError::Validation(msg) => {
                assert!(msg.contains("references undefined NPC 'missing_npc'"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_empty_npc_name_fails() {
        let mut parsed = valid_campaign();
        parsed.npcs.push(ParsedNpc {
            id: "guard".to_owned(),
            name: "  ".to_owned(),
            disposition: None,
        });
        let err = validate_parsed_campaign(&parsed).unwrap_err();
        match err {
            DomainError::Validation(msg) => {
                assert!(msg.contains("NPC 'guard' must have a non-empty name"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_multiple_errors_collected() {
        let parsed = ParsedCampaign {
            front_matter: CampaignFrontMatter {
                title: String::new(),
                description: None,
                min_engine_version: None,
            },
            scenes: Vec::new(),
            npcs: Vec::new(),
        };
        let err = validate_parsed_campaign(&parsed).unwrap_err();
        match err {
            DomainError::Validation(msg) => {
                assert!(msg.contains("title must not be empty"));
                assert!(msg.contains("at least one scene"));
                assert!(msg.contains("; "));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
