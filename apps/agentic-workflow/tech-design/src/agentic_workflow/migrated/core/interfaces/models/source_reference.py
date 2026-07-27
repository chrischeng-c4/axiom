"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/models/source_reference.md`.

Migrated by batch `projection-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-models-source-reference"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/models/source_reference.md"
__legacy_projection_digest__ = "sha256:b05e3a72bc5dd3b93d4c0f90caec83994509deb15891561685600c9ae7439633"


def render_markdown() -> Annotated[str, "sha256:b05e3a72bc5dd3b93d4c0f90caec83994509deb15891561685600c9ae7439633"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-agentic-workflow-src-models-source-reference-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: core-concept-model-and-invariants\n    claim: core-concept-model-and-invariants\n    coverage: full\n    rationale: \"Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure.\"\n---\n\n# Standardized apps/agentic-workflow/src/models/source_reference.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/models/source_reference.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `SourceFailureMode` | apps/agentic-workflow/src/models/source_reference.rs | enum | pub | 28 |  |\n| `SourceReference` | apps/agentic-workflow/src/models/source_reference.rs | struct | pub | 54 |  |\n| `SourceReferenceAvailability` | apps/agentic-workflow/src/models/source_reference.rs | enum | pub | 19 |  |\n| `SourceReferenceKind` | apps/agentic-workflow/src/models/source_reference.rs | enum | pub | 8 |  |\n| `SourceReferencePolicy` | apps/agentic-workflow/src/models/source_reference.rs | struct | pub | 46 |  |\n| `SourceReferenceRequirement` | apps/agentic-workflow/src/models/source_reference.rs | struct | pub | 36 |  |\n| `SourceReferenceReview` | apps/agentic-workflow/src/models/source_reference.rs | struct | pub | 87 |  |\n| `SourceReviewFinding` | apps/agentic-workflow/src/models/source_reference.rs | struct | pub | 79 |  |\n| `SourceReviewSeverity` | apps/agentic-workflow/src/models/source_reference.rs | enum | pub | 71 |  |\n| `evaluate_source_references` | apps/agentic-workflow/src/models/source_reference.rs | function | pub | 94 | evaluate_source_references(     policy: &SourceReferencePolicy,     references: &[SourceReference],     implementation_citations: &[String], ) -> SourceReferenceReview |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-managed-markers -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/models/source_reference.rs -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/models/source_reference.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Whole-file source replay owns the source-reference policy models until a\n      narrower schema generator can produce the full behavior surface.\n```\n"
