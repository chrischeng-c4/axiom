"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/spec_alignment/models_preamble_source.md`.

Migrated by batch `projection-core-interfaces-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-spec-alignment-models-preamble-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/spec_alignment/models_preamble_source.md"
__legacy_projection_digest__ = "sha256:b9aba023290e03e51ecfe5f4470d35f5a8adfcf1fc18a23640cf0fdb6021162d"


def render_markdown() -> Annotated[str, "sha256:b9aba023290e03e51ecfe5f4470d35f5a8adfcf1fc18a23640cf0fdb6021162d"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-spec-alignment-models-preamble-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: existing-project-standardization\n    role: primary\n    gap: traceability-closure-gate\n    claim: traceability-closure-gate\n    coverage: full\n    rationale: \"Spec alignment interfaces implement TD/source annotation and coverage checks used by the traceability closure gate.\"\n---\n\n# Spec Alignment Models Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/spec_alignment/models.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `CheckResult` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 15 |  |\n| `CodeBlock` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 30 |  |\n| `CoverageEntry` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 45 |  |\n| `CoverageReport` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 59 |  |\n| `FileResult` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 79 |  |\n| `OrphanRequirementEntry` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 91 |  |\n| `SchemaStructMismatchEntry` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 103 |  |\n| `SectionAnnotation` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 117 |  |\n| `SpecAnnotation` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 130 |  |\n| `SpecDocument` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 146 |  |\n| `SpecSection` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 158 |  |\n| `UnspeccedFunction` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 175 |  |\n| `Violation` | apps/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 189 |  |\n| `ViolationKind` | apps/agentic-workflow/src/spec_alignment/models.rs | enum | pub | 221 |  |\n| `is_format_violation` | apps/agentic-workflow/src/spec_alignment/models.rs | function | pub | 251 | is_format_violation(&self) -> bool |\n## Source\n<!-- type: source lang: rust -->\n\n```rust\n//! Data types for spec alignment checking.\n//!\n//! Corresponds to the JSON Schema definitions in the check-alignment change spec:\n//! SpecDocument, SpecSection, CodeBlock, Violation, ViolationKind, FileResult, CheckResult.\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/spec_alignment/models.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<module-preamble>\"\n    description: \"Source template owns spec-alignment model module documentation.\"\n```\n"
