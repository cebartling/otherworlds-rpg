//! Content Authoring — campaign Markdown parser.
//!
//! Parses campaign source (Markdown with YAML front-matter) into
//! intermediate `ParsedCampaign` representation.

use otherworlds_core::error::DomainError;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use super::campaign_model::{
    CampaignFrontMatter, ParsedCampaign, ParsedChoice, ParsedNpc, ParsedScene,
};

/// Extracts YAML front-matter from campaign source.
///
/// Expects the source to begin with `---\n`, followed by YAML content,
/// followed by `---\n`. Returns the parsed front-matter and the remaining
/// Markdown body.
///
/// # Errors
///
/// Returns `DomainError::Validation` if front-matter delimiters are missing
/// or YAML parsing fails.
pub fn extract_front_matter(source: &str) -> Result<(CampaignFrontMatter, &str), DomainError> {
    let trimmed = source.trim_start();
    if !trimmed.starts_with("---") {
        return Err(DomainError::Validation(
            "campaign source must begin with YAML front-matter (---)".to_owned(),
        ));
    }

    // Find the closing `---` delimiter (skip the opening line).
    let after_opening = &trimmed[3..];
    let after_opening = after_opening.strip_prefix('\n').unwrap_or(after_opening);

    let closing_pos = after_opening.find("\n---").ok_or_else(|| {
        DomainError::Validation("missing closing front-matter delimiter (---)".to_owned())
    })?;

    let yaml_content = &after_opening[..closing_pos];
    let body_start = closing_pos + 4; // skip "\n---"
    let body = if body_start < after_opening.len() {
        // Skip optional newline after closing ---
        let rest = &after_opening[body_start..];
        rest.strip_prefix('\n').unwrap_or(rest)
    } else {
        ""
    };

    let front_matter: CampaignFrontMatter = serde_yaml::from_str(yaml_content)
        .map_err(|e| DomainError::Validation(format!("invalid YAML front-matter: {e}")))?;

    Ok((front_matter, body))
}

/// Current parsing context within the Markdown document.
#[derive(Debug, PartialEq)]
enum SectionKind {
    /// Inside a `# Scene: <id>` block, collecting narrative text.
    SceneNarrative,
    /// Inside a `## Choices` sub-section of a scene.
    SceneChoices,
    /// Inside a `## NPCs` sub-section of a scene.
    SceneNpcRefs,
    /// Inside a `# NPC: <id>` block.
    NpcDefinition,
}

/// Parses the full campaign source into a `ParsedCampaign`.
///
/// # Errors
///
/// Returns `DomainError::Validation` if front-matter is missing or invalid.
#[allow(clippy::too_many_lines)]
pub fn parse_campaign(source: &str) -> Result<ParsedCampaign, DomainError> {
    let (front_matter, body) = extract_front_matter(source)?;

    let mut scenes: Vec<ParsedScene> = Vec::new();
    let mut npcs: Vec<ParsedNpc> = Vec::new();
    let mut current_section: Option<SectionKind> = None;

    let parser = Parser::new_ext(body, Options::empty());
    let events: Vec<Event<'_>> = parser.collect();
    let mut i = 0;

    while i < events.len() {
        match &events[i] {
            Event::Start(Tag::Heading { level, .. }) => {
                // Collect heading text
                i += 1;
                let mut heading_text = String::new();
                while i < events.len() {
                    match &events[i] {
                        Event::End(TagEnd::Heading(..)) => break,
                        Event::Text(t) => heading_text.push_str(t),
                        Event::Code(c) => heading_text.push_str(c),
                        _ => {}
                    }
                    i += 1;
                }

                match level {
                    HeadingLevel::H1 => {
                        if let Some(scene_id) = heading_text.strip_prefix("Scene:") {
                            let scene_id = scene_id.trim().to_owned();
                            scenes.push(ParsedScene {
                                id: scene_id,
                                narrative_text: String::new(),
                                choices: Vec::new(),
                                npc_refs: Vec::new(),
                            });
                            current_section = Some(SectionKind::SceneNarrative);
                        } else if let Some(npc_id) = heading_text.strip_prefix("NPC:") {
                            let npc_id = npc_id.trim().to_owned();
                            npcs.push(ParsedNpc {
                                id: npc_id,
                                name: String::new(),
                                disposition: None,
                            });
                            current_section = Some(SectionKind::NpcDefinition);
                        } else {
                            current_section = None;
                        }
                    }
                    HeadingLevel::H2 => {
                        let trimmed_heading = heading_text.trim();
                        if trimmed_heading == "Choices" {
                            current_section = Some(SectionKind::SceneChoices);
                        } else if trimmed_heading == "NPCs" {
                            current_section = Some(SectionKind::SceneNpcRefs);
                        } else {
                            // Unknown H2 — stay in scene narrative if we were there.
                        }
                    }
                    _ => {}
                }
            }

            // Collect paragraph text as narrative for scene narrative sections.
            Event::Start(Tag::Paragraph)
                if current_section == Some(SectionKind::SceneNarrative) =>
            {
                i += 1;
                let mut para_text = String::new();
                while i < events.len() {
                    match &events[i] {
                        Event::End(TagEnd::Paragraph) => break,
                        Event::Text(t) => para_text.push_str(t),
                        Event::SoftBreak | Event::HardBreak => para_text.push('\n'),
                        Event::Code(c) => para_text.push_str(c),
                        _ => {}
                    }
                    i += 1;
                }
                if let Some(scene) = scenes.last_mut() {
                    if !scene.narrative_text.is_empty() {
                        scene.narrative_text.push_str("\n\n");
                    }
                    scene.narrative_text.push_str(&para_text);
                }
            }

            // Parse list items in Choices section — expect `[label](scene:target)`.
            Event::Start(Tag::Item) if current_section == Some(SectionKind::SceneChoices) => {
                i += 1;
                while i < events.len() {
                    match &events[i] {
                        Event::End(TagEnd::Item) => break,
                        Event::Start(Tag::Link { dest_url, .. }) => {
                            let dest = dest_url.to_string();
                            // Collect link text
                            i += 1;
                            let mut link_text = String::new();
                            while i < events.len() {
                                match &events[i] {
                                    Event::End(TagEnd::Link) => break,
                                    Event::Text(t) => link_text.push_str(t),
                                    _ => {}
                                }
                                i += 1;
                            }
                            if let Some(target) = dest.strip_prefix("scene:")
                                && let Some(scene) = scenes.last_mut()
                            {
                                scene.choices.push(ParsedChoice {
                                    label: link_text,
                                    target: target.to_owned(),
                                });
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }
            }

            // Parse list items in NPC refs section — plain text NPC IDs.
            Event::Start(Tag::Item) if current_section == Some(SectionKind::SceneNpcRefs) => {
                i += 1;
                let mut item_text = String::new();
                while i < events.len() {
                    match &events[i] {
                        Event::End(TagEnd::Item) => break,
                        Event::Text(t) => item_text.push_str(t),
                        _ => {}
                    }
                    i += 1;
                }
                let npc_ref = item_text.trim().to_owned();
                if !npc_ref.is_empty()
                    && let Some(scene) = scenes.last_mut()
                {
                    scene.npc_refs.push(npc_ref);
                }
            }

            // Parse list items in NPC definition section — `name:` and `disposition:` properties.
            Event::Start(Tag::Item) if current_section == Some(SectionKind::NpcDefinition) => {
                i += 1;
                let mut item_text = String::new();
                while i < events.len() {
                    match &events[i] {
                        Event::End(TagEnd::Item) => break,
                        Event::Text(t) => item_text.push_str(t),
                        _ => {}
                    }
                    i += 1;
                }
                let item_trimmed = item_text.trim();
                if let Some(name_val) = item_trimmed.strip_prefix("name:") {
                    if let Some(npc) = npcs.last_mut() {
                        name_val.trim().clone_into(&mut npc.name);
                    }
                } else if let Some(disp_val) = item_trimmed.strip_prefix("disposition:")
                    && let Some(npc) = npcs.last_mut()
                {
                    npc.disposition = Some(disp_val.trim().to_owned());
                }
            }

            _ => {}
        }
        i += 1;
    }

    Ok(ParsedCampaign {
        front_matter,
        scenes,
        npcs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_front_matter_happy_path() {
        let source =
            "---\ntitle: \"Test Campaign\"\ndescription: \"A test\"\n---\n\n# Scene: start\n";
        let (fm, body) = extract_front_matter(source).unwrap();
        assert_eq!(fm.title, "Test Campaign");
        assert_eq!(fm.description, Some("A test".to_owned()));
        assert!(body.trim_start().starts_with("# Scene: start"));
    }

    #[test]
    fn test_extract_front_matter_missing_opening() {
        let source = "# No front matter here\n";
        let result = extract_front_matter(source);
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => assert!(msg.contains("front-matter")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_extract_front_matter_missing_closing() {
        let source = "---\ntitle: \"Test\"\n";
        let result = extract_front_matter(source);
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => assert!(msg.contains("closing")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_extract_front_matter_invalid_yaml() {
        let source = "---\n: invalid: yaml: [[\n---\n";
        let result = extract_front_matter(source);
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => assert!(msg.contains("YAML")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_single_scene() {
        let source = "---\ntitle: \"Test\"\n---\n\n# Scene: start\n\nHello world.\n";
        let parsed = parse_campaign(source).unwrap();
        assert_eq!(parsed.scenes.len(), 1);
        assert_eq!(parsed.scenes[0].id, "start");
        assert_eq!(parsed.scenes[0].narrative_text, "Hello world.");
    }

    #[test]
    fn test_parse_scene_with_choices() {
        let source = concat!(
            "---\ntitle: \"Test\"\n---\n\n",
            "# Scene: entrance\n\n",
            "Welcome.\n\n",
            "## Choices\n",
            "- [Enter the temple](scene:inner_hall)\n",
            "- [Search around](scene:perimeter)\n",
        );
        let parsed = parse_campaign(source).unwrap();
        assert_eq!(parsed.scenes.len(), 1);
        let scene = &parsed.scenes[0];
        assert_eq!(scene.choices.len(), 2);
        assert_eq!(scene.choices[0].label, "Enter the temple");
        assert_eq!(scene.choices[0].target, "inner_hall");
        assert_eq!(scene.choices[1].label, "Search around");
        assert_eq!(scene.choices[1].target, "perimeter");
    }

    #[test]
    fn test_parse_scene_with_npc_refs() {
        let source = concat!(
            "---\ntitle: \"Test\"\n---\n\n",
            "# Scene: start\n\n",
            "A room.\n\n",
            "## NPCs\n",
            "- guard_captain\n",
            "- merchant\n",
        );
        let parsed = parse_campaign(source).unwrap();
        assert_eq!(parsed.scenes[0].npc_refs, vec!["guard_captain", "merchant"]);
    }

    #[test]
    fn test_parse_npc_definitions() {
        let source = concat!(
            "---\ntitle: \"Test\"\n---\n\n",
            "# Scene: start\n\nHello.\n\n",
            "# NPC: guard_captain\n\n",
            "- name: Captain Theron\n",
            "- disposition: neutral\n",
        );
        let parsed = parse_campaign(source).unwrap();
        assert_eq!(parsed.npcs.len(), 1);
        assert_eq!(parsed.npcs[0].id, "guard_captain");
        assert_eq!(parsed.npcs[0].name, "Captain Theron");
        assert_eq!(parsed.npcs[0].disposition, Some("neutral".to_owned()));
    }

    #[test]
    fn test_parse_full_campaign() {
        let source = concat!(
            "---\ntitle: \"The Lost Temple\"\ndescription: \"An adventure\"\nmin_engine_version: 1\n---\n\n",
            "# Scene: entrance\n\n",
            "The ancient temple entrance looms before you...\n\n",
            "## Choices\n",
            "- [Enter the temple](scene:inner_hall)\n",
            "- [Search the perimeter](scene:perimeter)\n\n",
            "## NPCs\n",
            "- guard_captain\n\n",
            "# Scene: inner_hall\n\n",
            "The inner hall is dark and musty...\n\n",
            "## Choices\n",
            "- [Go back](scene:entrance)\n\n",
            "# Scene: perimeter\n\n",
            "You circle the exterior...\n\n",
            "# NPC: guard_captain\n\n",
            "- name: Captain Theron\n",
            "- disposition: neutral\n",
        );
        let parsed = parse_campaign(source).unwrap();
        assert_eq!(parsed.front_matter.title, "The Lost Temple");
        assert_eq!(
            parsed.front_matter.description,
            Some("An adventure".to_owned())
        );
        assert_eq!(parsed.front_matter.min_engine_version, Some(1));
        assert_eq!(parsed.scenes.len(), 3);
        assert_eq!(parsed.npcs.len(), 1);
        assert_eq!(parsed.scenes[0].choices.len(), 2);
        assert_eq!(parsed.scenes[1].choices.len(), 1);
        assert_eq!(parsed.scenes[0].npc_refs, vec!["guard_captain"]);
    }

    #[test]
    fn test_parse_multiple_scenes_preserve_order() {
        let source = concat!(
            "---\ntitle: \"Test\"\n---\n\n",
            "# Scene: alpha\n\nFirst.\n\n",
            "# Scene: beta\n\nSecond.\n\n",
            "# Scene: gamma\n\nThird.\n",
        );
        let parsed = parse_campaign(source).unwrap();
        assert_eq!(parsed.scenes.len(), 3);
        assert_eq!(parsed.scenes[0].id, "alpha");
        assert_eq!(parsed.scenes[1].id, "beta");
        assert_eq!(parsed.scenes[2].id, "gamma");
    }
}
