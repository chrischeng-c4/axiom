"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/spec_alignment/format_rules_runtime_source.md`.

Migrated by batch `projection-core-interfaces-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-spec-alignment-format-rules-runtime-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/spec_alignment/format_rules_runtime_source.md"
__legacy_projection_digest__ = "sha256:21bc158a9338fabee42424bf471d5bebe0ee23982a5d50b021f8e080c6931e4e"


def render_markdown() -> Annotated[str, "sha256:21bc158a9338fabee42424bf471d5bebe0ee23982a5d50b021f8e080c6931e4e"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-spec-alignment-format-rules-runtime-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: existing-project-standardization\n    role: primary\n    gap: traceability-closure-gate\n    claim: traceability-closure-gate\n    coverage: full\n    rationale: \"Spec alignment interfaces implement TD/source annotation and coverage checks used by the traceability closure gate.\"\n---\n\n# Spec Alignment Format Rules Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/spec_alignment/format_rules.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `check` | apps/agentic-workflow/src/spec_alignment/format_rules.rs | function | pub | 63 | check(doc: &SpecDocument) -> Vec<Violation> |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap spec-alignment-format-rules-runtime -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/spec_alignment/format_rules.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:spec-alignment-format-rules-runtime>\"\n    description: \"Source template owns spec-alignment format-rule runtime behavior and tests.\"\n```\n"
