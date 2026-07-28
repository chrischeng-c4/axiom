"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/parser/mod.md`.

Migrated by batch `projection-core-interfaces-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-parser-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/parser/mod.md"
__legacy_projection_digest__ = "sha256:f8ac70c1fd3991c77775ff6dcc7824fe317d5e8d2562547ae0dfc3fce4d390d5"


def render_markdown() -> Annotated[str, "sha256:f8ac70c1fd3991c77775ff6dcc7824fe317d5e8d2562547ae0dfc3fce4d390d5"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-parser-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: core-concept-model-and-invariants\n    claim: core-concept-model-and-invariants\n    coverage: full\n    rationale: \"Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure.\"\n---\n\n# Standardized apps/agentic-workflow/src/parser/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/parser/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `archive_review` | apps/agentic-workflow/src/parser/mod.rs | module | pub | 3 |  |\n| `challenge` | apps/agentic-workflow/src/parser/mod.rs | module | pub | 4 |  |\n| `frontmatter` | apps/agentic-workflow/src/parser/mod.rs | module | pub | 5 |  |\n| `inline_yaml` | apps/agentic-workflow/src/parser/mod.rs | module | pub | 6 |  |\n| `markdown` | apps/agentic-workflow/src/parser/mod.rs | module | pub | 7 |  |\n| `requirement` | apps/agentic-workflow/src/parser/mod.rs | module | pub | 8 |  |\n| `review` | apps/agentic-workflow/src/parser/mod.rs | module | pub | 9 |  |\n| `scenario` | apps/agentic-workflow/src/parser/mod.rs | module | pub | 10 |  |\n| `xml` | apps/agentic-workflow/src/parser/mod.rs | module | pub | 11 |  |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-handwrite -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/parser/mod.rs -->\n```rust\npub mod archive_review;\npub mod challenge;\npub mod frontmatter;\npub mod inline_yaml;\npub mod markdown;\npub mod requirement;\npub mod review;\npub mod scenario;\npub mod xml;\n\npub use archive_review::{get_review_path, parse_archive_review_verdict};\npub use challenge::{parse_challenge_verdict, ChallengeParser};\npub use frontmatter::{\n    calculate_body_checksum, calculate_checksum, has_frontmatter, is_stale, normalize_content,\n    parse_document, parse_frontmatter_value, split_frontmatter, ParsedDocument,\n};\npub use inline_yaml::{\n    extract_yaml_blocks, extract_yaml_blocks_with_lines, parse_issue_blocks,\n    parse_requirement_blocks, parse_task_blocks, parse_typed_yaml_blocks, YamlBlock,\n};\npub use markdown::extract_heading_section;\npub use requirement::RequirementParser;\npub use review::{parse_latest_review, parse_review_verdict, ReviewBlock};\npub use scenario::ScenarioParser;\npub use xml::{\n    extract_xml_block, extract_xml_blocks, parse_xml_attributes, update_xml_blocks, wrap_in_xml,\n    UpdateMode, XmlBlock,\n};\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/parser/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Source template owns the complete parser module facade.\n```\n"
