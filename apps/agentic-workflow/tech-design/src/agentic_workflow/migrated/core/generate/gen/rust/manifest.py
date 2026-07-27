"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/gen/rust/manifest.md`.

Migrated by batch `projection-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-gen-rust-manifest"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/gen/rust/manifest.md"
__legacy_projection_digest__ = "sha256:2eb17a762f85c087133b22bb334bae523067dc6526fc0fdd514141c54ade26ff"


def render_markdown() -> Annotated[str, "sha256:2eb17a762f85c087133b22bb334bae523067dc6526fc0fdd514141c54ade26ff"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-generate-gen-rust-manifest-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Manifest Generator\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/gen/rust/manifest.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `ManifestGenOutput` | apps/agentic-workflow/src/generate/gen/rust/manifest.rs | struct | pub | 43 |  |\n| `generate_manifest` | apps/agentic-workflow/src/generate/gen/rust/manifest.rs | function | pub | 58 | generate_manifest(spec_content: &str) -> ManifestGenOutput |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-managed-markers -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/gen/rust/manifest.rs -->\n````rust\n//! Manifest generator — Cargo.toml `[dependencies]` fragment from a `manifest`\n//! section.\n//!\n//! Section contract (YAML inside the spec's `## Manifest` section):\n//!\n//! ```yaml\n//! dependencies:\n//!   - { name: serde, spec: workspace, features: [derive] }\n//!   - { name: thiserror, spec: workspace }\n//!   - { name: once_cell, spec: version, version: \"1.20\" }\n//!   - { name: cclab-mamba-registry, spec: path, path: \"../../crates/cclab-mamba-registry\" }\n//! ```\n//!\n//! Output is a TOML fragment (one `key = value` per line) suitable for wrapping\n//! inside a CODEGEN block under `[dependencies]` in the target `Cargo.toml`:\n//!\n//! ```toml\n//! [dependencies]\n//! serde = { workspace = true, features = [\"derive\"] }\n//! thiserror.workspace = true\n//! once_cell = { version = \"1.20\" }\n//! cclab-mamba-registry = { path = \"../../crates/cclab-mamba-registry\" }\n````\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/gen/rust/manifest.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Source template owns the complete manifest generator module.\n```\n"
