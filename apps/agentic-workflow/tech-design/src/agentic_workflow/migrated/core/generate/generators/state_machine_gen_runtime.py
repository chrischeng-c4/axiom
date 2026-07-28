"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/generators/state_machine_gen_runtime.md`.

Migrated by batch `projection-core-generate-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-state-machine-gen-runtime"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/state_machine_gen_runtime.md"
__legacy_projection_digest__ = "sha256:e11dcd8d8238ae882a520dc5bb21affca7d6b4bbfbe22f92f0d3e64103d77ff8"


def render_markdown() -> Annotated[str, "sha256:e11dcd8d8238ae882a520dc5bb21affca7d6b4bbfbe22f92f0d3e64103d77ff8"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-generators-state-machine-gen-runtime\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# StateMachineGenerator Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/generators/state_machine_gen.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `StateMachineGenerator` | apps/agentic-workflow/src/generate/generators/state_machine_gen.rs | struct | pub | 29 |  |\n| `new` | apps/agentic-workflow/src/generate/generators/state_machine_gen.rs | function | pub | 37 | new() -> Self |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap generate-generators-state-machine-gen-runtime -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/generators/state_machine_gen.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:generate-generators-state-machine-gen-runtime>\"\n    description: \"Source template owns the state machine generator runtime and regression tests.\"\n```\n"
