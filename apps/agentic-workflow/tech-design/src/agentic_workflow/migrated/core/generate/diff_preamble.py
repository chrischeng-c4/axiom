"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/diff_preamble.md`.

Migrated by batch `projection-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-diff-preamble"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/diff_preamble.md"
__legacy_projection_digest__ = "sha256:a5eaf0a3a6e8e813b8a3c645d90b7c356f708ab9a4a1ef172b3cc5297dbdf0bd"


def render_markdown() -> Annotated[str, "sha256:a5eaf0a3a6e8e813b8a3c645d90b7c356f708ab9a4a1ef172b3cc5297dbdf0bd"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-diff-preamble\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Diff Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/diff.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `DiffClass` | apps/agentic-workflow/src/generate/diff.rs | enum | pub | 23 |  |\n| `DiffReport` | apps/agentic-workflow/src/generate/diff.rs | struct | pub | 37 |  |\n| `FileDiff` | apps/agentic-workflow/src/generate/diff.rs | struct | pub | 45 |  |\n| `has_drift` | apps/agentic-workflow/src/generate/diff.rs | function | pub | 74 | has_drift(&self) -> bool |\n| `overall_drift_pct` | apps/agentic-workflow/src/generate/diff.rs | function | pub | 65 | overall_drift_pct(&self) -> f32 |\n| `run_diff` | apps/agentic-workflow/src/generate/diff.rs | function | pub | 87 | run_diff(spec_path: &Path, project_root: &Path) -> crate::generate::Result<DiffReport> |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap generate-diff-preamble -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/diff.rs -->\n```rust\n\n//! Diff implementation: compare current target files against what codegen would produce.\n//!\n//! `run_diff` runs codegen for a spec, compares the generated CODEGEN block content\n//! against what is currently in the target file, and classifies the difference.\n//!\n//! Classification:\n//! - `Exact`: Current content matches generated content (no drift)\n//! - `MarkerOnly`: CODEGEN markers present but empty content\n//! - `Drift`: Content differs from generated output\n//! - `Gap`: No CODEGEN markers found in the target file\n\n// @spec apps/agentic-workflow/tech-design/core/logic/codegen-validation.md\n\nuse std::path::{Path, PathBuf};\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/diff.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:generate-diff-preamble>\"\n    description: \"Source template owns diff module docs and imports.\"\n```\n"
