"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/spec_alignment/models_impl_source.md`.

Migrated by batch `projection-core-interfaces-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-spec-alignment-models-impl-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/spec_alignment/models_impl_source.md"
__legacy_projection_digest__ = "sha256:51ff0b6ba52a1b7a67f5931c69a3d441f0a611b7641f506ba586952e88c77979"


def render_markdown() -> Annotated[str, "sha256:51ff0b6ba52a1b7a67f5931c69a3d441f0a611b7641f506ba586952e88c77979"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-spec-alignment-models-impl-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: existing-project-standardization\n    role: primary\n    gap: traceability-closure-gate\n    claim: traceability-closure-gate\n    coverage: full\n    rationale: \"Spec alignment interfaces implement TD/source annotation and coverage checks used by the traceability closure gate.\"\n---\n\n# Spec Alignment Models Impl Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/spec_alignment/models.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `CheckResult` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 15 |  |\n| `CodeBlock` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 30 |  |\n| `CoverageEntry` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 45 |  |\n| `CoverageReport` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 59 |  |\n| `FileResult` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 79 |  |\n| `OrphanRequirementEntry` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 91 |  |\n| `SchemaStructMismatchEntry` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 103 |  |\n| `SectionAnnotation` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 117 |  |\n| `SpecAnnotation` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 130 |  |\n| `SpecDocument` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 146 |  |\n| `SpecSection` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 158 |  |\n| `UnspeccedFunction` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 175 |  |\n| `Violation` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 189 |  |\n| `ViolationKind` | apps/agentic-workflow/src/spec_alignment/models.rs | enum | pub | 221 |  |\n| `is_format_violation` | apps/agentic-workflow/src/spec_alignment/models.rs | function | pub | 251 | is_format_violation(&self) -> bool |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap spec-alignment-model-impls -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/spec_alignment/models.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:spec-alignment-model-impls>\"\n    description: \"Source template owns ViolationKind helper and Display impls.\"\n```\n"
