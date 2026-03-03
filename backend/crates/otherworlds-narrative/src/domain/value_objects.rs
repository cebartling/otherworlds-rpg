//! Value objects for the Narrative Orchestration context.

use serde::{Deserialize, Serialize};

/// A choice option presented to the player within a scene.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChoiceOption {
    /// The display label for this choice.
    pub label: String,
    /// The scene ID this choice transitions to.
    pub target_scene_id: String,
}

/// Scene data passed into the narrative context from the content layer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneData {
    /// The scene identifier (author-defined, e.g. "tavern", "start").
    pub scene_id: String,
    /// The narrative text displayed to the player.
    pub narrative_text: String,
    /// The choices available in this scene.
    pub choices: Vec<ChoiceOption>,
    /// NPC references present in this scene.
    pub npc_refs: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choice_option_json_round_trip() {
        let choice = ChoiceOption {
            label: "Enter the tavern".to_owned(),
            target_scene_id: "tavern".to_owned(),
        };

        let json = serde_json::to_value(&choice).unwrap();
        let deserialized: ChoiceOption = serde_json::from_value(json).unwrap();

        assert_eq!(choice, deserialized);
    }

    #[test]
    fn test_scene_data_json_round_trip() {
        let scene = SceneData {
            scene_id: "start".to_owned(),
            narrative_text: "You stand at the crossroads.".to_owned(),
            choices: vec![
                ChoiceOption {
                    label: "Go north".to_owned(),
                    target_scene_id: "forest".to_owned(),
                },
                ChoiceOption {
                    label: "Go south".to_owned(),
                    target_scene_id: "village".to_owned(),
                },
            ],
            npc_refs: vec!["old_sage".to_owned()],
        };

        let json = serde_json::to_value(&scene).unwrap();
        let deserialized: SceneData = serde_json::from_value(json).unwrap();

        assert_eq!(scene, deserialized);
    }
}
