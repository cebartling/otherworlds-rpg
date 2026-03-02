# ADR-0012: Campaign Markdown Format

## Status

Accepted

## Context

The Content Authoring bounded context ingests campaign source as raw strings but has no defined format for parsing, validating, or compiling that source into structured runtime data. The `validate_campaign()` and `compile_campaign()` methods on the `Campaign` aggregate are stubs that emit events without performing real work.

We need a concrete, human-readable authoring format that:
- Is easy for content authors to write and version-control
- Can be validated for structural correctness (scene references, NPC references)
- Can be compiled into an indexed runtime structure for O(1) scene/NPC lookup

## Decision

Campaign source uses **Markdown with YAML front-matter**:

- `---` fenced YAML front-matter provides campaign metadata (`title` required, `description` and `min_engine_version` optional).
- `# Scene: <id>` H1 headings define scenes with narrative body text.
- `## Choices` H2 sub-sections within scenes contain Markdown links `[label](scene:target_id)` for scene transitions.
- `## NPCs` H2 sub-sections within scenes list NPC ID references.
- `# NPC: <id>` H1 headings define NPCs with `- name:` and `- disposition:` properties.

Parsing uses `pulldown-cmark` (CommonMark) for Markdown and `serde_yaml` for YAML front-matter.

Validation enforces seven rules:
1. Non-empty title in front-matter
2. At least one scene defined
3. No duplicate scene IDs
4. No duplicate NPC IDs
5. All choice targets reference defined scene IDs
6. All NPC refs in scenes reference defined NPC IDs
7. Every NPC must have a non-empty name

Compilation converts `ParsedCampaign` (vec-based) into `CompiledCampaign` (HashMap-indexed) and serializes to JSON for storage in the `CampaignCompiled` event.

## Consequences

- **Easier**: Content authors write standard Markdown. Validation catches broken references before compilation. Compiled data enables fast runtime lookup.
- **Harder**: The format is opinionated — adding new structural elements (e.g., items, quests) requires extending the parser and validator. CommonMark is a subset of what authors might expect from full GFM.
- **Dependencies**: Adds `pulldown-cmark` and `serde_yaml` to the workspace.
