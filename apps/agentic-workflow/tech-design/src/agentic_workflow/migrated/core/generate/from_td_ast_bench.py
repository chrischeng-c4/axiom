"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/from-td-ast-bench.md`.

Migrated by batch `semantic-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-from-td-ast-bench"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/from-td-ast-bench.md"
__legacy_td_digest__ = "sha256:937fbb6445af2358de79e678196c21c5b38a65334a392b18076d04863cfd938e"


def render_markdown() -> Annotated[str, "sha256:937fbb6445af2358de79e678196c21c5b38a65334a392b18076d04863cfd938e"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-generate-from-td-ast-bench\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# TDAst Dispatch Bench Stub\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/benches/dispatch_perf.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\nNo public AST symbols.\n## Source\n<!-- type: source lang: rust -->\n\n```rust\n//! Bench scaffolding for R10: TDAst-based dispatch performance.\n//!\n//! Stage 2 keeps this as a stub - the perf comparison vs. legacy dispatch\n//! lands once Stage 2B migrates the generators end-to-end. Compiles as a\n//! `--bin` so cargo doesn't need a bench target wired up.\n//!\n//! @spec apps/agentic-workflow/tech-design/core/generate/from-td-ast.md#logic\n\nfn main() {\n    println!(\"dispatch_perf: bench scaffolding only; see Stage 2B follow-up.\");\n}\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/benches/dispatch_perf.rs\n    action: create\n    section: source\n    impl_mode: codegen\n    description: \"Bench harness stub for the future TDAst dispatch performance comparison.\"\n```\n"
