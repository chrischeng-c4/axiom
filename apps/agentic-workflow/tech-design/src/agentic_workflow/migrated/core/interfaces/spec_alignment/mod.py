"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/spec_alignment/mod.md`.

Migrated by batch `projection-core-interfaces-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-spec-alignment-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/spec_alignment/mod.md"
__legacy_projection_digest__ = "sha256:a070e4d37896137a1540197f663ec4f22a381cc33ff6f88f539f02b5e26f2527"


def render_markdown() -> Annotated[str, "sha256:a070e4d37896137a1540197f663ec4f22a381cc33ff6f88f539f02b5e26f2527"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-spec-alignment-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: existing-project-standardization\n    role: primary\n    gap: traceability-closure-gate\n    claim: traceability-closure-gate\n    coverage: full\n    rationale: \"Spec alignment interfaces implement TD/source annotation and coverage checks used by the traceability closure gate.\"\n---\n\n# Standardized apps/agentic-workflow/src/spec_alignment/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/spec_alignment/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `annotations` | apps/agentic-workflow/src/spec_alignment/mod.rs | module | pub | 12 |  |\n| `check` | apps/agentic-workflow/src/spec_alignment/mod.rs | module | pub | 13 |  |\n| `coverage` | apps/agentic-workflow/src/spec_alignment/mod.rs | module | pub | 14 |  |\n| `format_rules` | apps/agentic-workflow/src/spec_alignment/mod.rs | module | pub | 15 |  |\n| `logical_rules` | apps/agentic-workflow/src/spec_alignment/mod.rs | module | pub | 16 |  |\n| `models` | apps/agentic-workflow/src/spec_alignment/mod.rs | module | pub | 17 |  |\n| `parser` | apps/agentic-workflow/src/spec_alignment/mod.rs | module | pub | 18 |  |\n| `requirement_coverage` | apps/agentic-workflow/src/spec_alignment/mod.rs | module | pub | 19 |  |\n| `schema_struct` | apps/agentic-workflow/src/spec_alignment/mod.rs | module | pub | 20 |  |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap standardize:claim-code -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/spec_alignment/mod.rs -->\n```rust\n//! Spec alignment checking.\n//!\n//! Validates spec files for format compliance and logical consistency.\n//! Two-layer validation:\n//! - Format compliance: section annotations, duplicates, code block requirements\n//! - Logical consistency: duplicate definitions, schema conflicts, field near-matches\n//!\n//! Entry point: `spec_alignment::check(path)`.\n\npub mod annotations;\npub mod check;\npub mod coverage;\npub mod format_rules;\npub mod logical_rules;\npub mod models;\npub mod parser;\npub mod requirement_coverage;\npub mod schema_struct;\n\npub use check::{check, check_with_coverage};\npub use models::{\n    CheckResult, CodeBlock, CoverageEntry, CoverageReport, FileResult, OrphanRequirementEntry,\n    SchemaStructMismatchEntry, SpecAnnotation, SpecDocument, SpecSection, UnspeccedFunction,\n    Violation, ViolationKind,\n};\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/spec_alignment/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:standardize:claim-code>\"\n    description: |\n      Source template owns the complete spec-alignment module facade.\n```\n"
