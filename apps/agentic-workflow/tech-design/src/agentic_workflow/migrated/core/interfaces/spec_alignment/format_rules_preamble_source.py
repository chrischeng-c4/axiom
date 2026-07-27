"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/spec_alignment/format_rules_preamble_source.md`.

Migrated by batch `projection-core-interfaces-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-spec-alignment-format-rules-preamble-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/spec_alignment/format_rules_preamble_source.md"
__legacy_projection_digest__ = "sha256:301342b878b95b9623df13d68b906337fc63b315c1da815b7dce39326f7fa0f6"


def render_markdown() -> Annotated[str, "sha256:301342b878b95b9623df13d68b906337fc63b315c1da815b7dce39326f7fa0f6"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-spec-alignment-format-rules-preamble-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: existing-project-standardization\n    role: primary\n    gap: traceability-closure-gate\n    claim: traceability-closure-gate\n    coverage: full\n    rationale: \"Spec alignment interfaces implement TD/source annotation and coverage checks used by the traceability closure gate.\"\n---\n\n# Spec Alignment Format Rules Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/spec_alignment/format_rules.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `check` | apps/agentic-workflow/src/spec_alignment/format_rules.rs | function | pub | 63 | check(doc: &SpecDocument) -> Vec<Violation> |\n## Source\n<!-- type: source lang: rust -->\n\n```rust\n//! Format compliance rules for spec alignment checking.\n//!\n//! Three rules:\n//! - `missing_section_annotation`: every `## Heading` must have an annotation\n//! - `duplicate_section`: no duplicate heading text within a file\n//! - `format_priority_violation`: typed sections must contain matching code blocks\n\nuse std::collections::HashMap;\n\n#[cfg(test)]\nuse super::models::{CodeBlock, SectionAnnotation, SpecSection};\nuse super::models::{SpecDocument, Violation, ViolationKind};\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/spec_alignment/format_rules.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<module-preamble>\"\n    description: \"Source template owns spec-alignment format-rules documentation and imports.\"\n```\n"
