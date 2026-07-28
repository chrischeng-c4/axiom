"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/spec_ir/mod.md`.

Migrated by batch `projection-core-generate-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-spec-ir-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/spec_ir/mod.md"
__legacy_projection_digest__ = "sha256:f3b0ef73e7ed4a9db3731d88a4e5e9429b9e2716bed87f592711a1d8f9432118"


def render_markdown() -> Annotated[str, "sha256:f3b0ef73e7ed4a9db3731d88a4e5e9429b9e2716bed87f592711a1d8f9432118"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-generate-spec-ir-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Standardized apps/agentic-workflow/src/generate/spec_ir/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/spec_ir/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\nNo public AST symbols.\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-managed-markers -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/spec_ir/mod.rs -->\n```rust\n//! SpecIR — Specification Intermediate Representation\n//!\n//! The universal contract between SDD generate (spec format) and Lens (code generation).\n//! SpecIR wraps diagram and schema types into a unified enum that\n//! generators can consume via `can_generate()` / `generate_from_ir()`.\n//!\n//! ## Variants\n//!\n//! | Variant | Section type | Generator |\n//! |---------|-------------|-----------|\n//! | `Api` | `rest-api` / `schema` | `FastAPIGenerator`, `ExpressGenerator`, `AxumGenerator` |\n//! | `FlowchartPlus` | `logic` (flowchart) | — |\n//! | `ClassPlus` | `logic` (class) | — |\n//! | `ErdPlus` | `db-model` | — |\n//! | `SequencePlus` | `interaction` | — |\n//! | `RequirementPlus` | `test-plan` | `TestGenerator` |\n//! | `Deploy` | `deploy` | `DeployGenerator` |\n//! | `Wireframe` | `wireframe` | `ReactGenerator` |\n//! | `Component` | `component` | — (future) |\n//! | `DesignToken` | `design-token` | — (future) |\n\nmod types;\n\npub use types::*;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/spec_ir/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: Source template owns the complete SpecIR module facade.\n```\n"
