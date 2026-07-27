"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/state/mod.md`.

Migrated by batch `projection-core-interfaces-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-state-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/state/mod.md"
__legacy_projection_digest__ = "sha256:b4e1f7e5988dec03f4c91e42913eb979b0f0183fdee618be02658a33b24781ba"


def render_markdown() -> Annotated[str, "sha256:b4e1f7e5988dec03f4c91e42913eb979b0f0183fdee618be02658a33b24781ba"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-state-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: core-concept-model-and-invariants\n    claim: core-concept-model-and-invariants\n    coverage: full\n    rationale: \"Workflow state interfaces define AW Core lifecycle state, locks, validations, and rollup invariants.\"\n---\n\n# Standardized apps/agentic-workflow/src/state/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/state/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\nNo public AST symbols.\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-handwrite -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/state/mod.rs -->\n```rust\n//! STATE.yaml Management Module\n//!\n//! Handles persistence and tracking of change state, including:\n//! - Phase transitions\n//! - File checksums for staleness detection\n//! - Validation history\n//! - LLM telemetry\n\nmod manager;\n\npub(crate) use manager::run_blocking_io;\npub use manager::{AgentLock, StalenessReport, StateManager};\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/state/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Source template owns the complete state module facade.\n```\n"
