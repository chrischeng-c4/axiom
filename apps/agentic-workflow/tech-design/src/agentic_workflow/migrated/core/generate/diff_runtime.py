"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/diff_runtime.md`.

Migrated by batch `projection-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-diff-runtime"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/diff_runtime.md"
__legacy_projection_digest__ = "sha256:430f57d57355de2478e427435a4875495d15cf008ce71eb08ffce39b4cd956be"


def render_markdown() -> Annotated[str, "sha256:430f57d57355de2478e427435a4875495d15cf008ce71eb08ffce39b4cd956be"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-diff-runtime\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Diff Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/diff.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `DiffClass` | apps/agentic-workflow/src/generate/diff.rs | enum | pub | 23 |  |\n| `DiffReport` | apps/agentic-workflow/src/generate/diff.rs | struct | pub | 37 |  |\n| `FileDiff` | apps/agentic-workflow/src/generate/diff.rs | struct | pub | 45 |  |\n| `has_drift` | apps/agentic-workflow/src/generate/diff.rs | function | pub | 74 | has_drift(&self) -> bool |\n| `overall_drift_pct` | apps/agentic-workflow/src/generate/diff.rs | function | pub | 65 | overall_drift_pct(&self) -> f32 |\n| `run_diff` | apps/agentic-workflow/src/generate/diff.rs | function | pub | 87 | run_diff(spec_path: &Path, project_root: &Path) -> crate::generate::Result<DiffReport> |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap generate-diff-runtime -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/diff.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:generate-diff-runtime>\"\n    description: \"Source template owns diff runtime helpers and regression tests.\"\n```\n"
