"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/fillback/factory.md`.

Migrated by batch `semantic-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-fillback-factory"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/fillback/factory.md"
__legacy_td_digest__ = "sha256:34a9fc1b6077d5dd9a5118134fc982baa528dd5bd5c1097ddbe44493b0e81ab7"


def render_markdown() -> Annotated[str, "sha256:34a9fc1b6077d5dd9a5118134fc982baa528dd5bd5c1097ddbe44493b0e81ab7"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-fillback-factory\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# StrategyFactory Type\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nFactory for creating import strategy instances in\n`apps/agentic-workflow/src/fillback/factory.rs`. One shape:\n\n- `StrategyFactory` — unit struct with no fields and no derives.\n  Acts as a namespace for the static `create` and private\n  `auto_detect` factory functions in the hand-written\n  `impl StrategyFactory` block.\n\nCodegen replaces the unit struct declaration. Companion source specs own the\nfactory imports and the runtime implementation/tests.\n\nThis spec exercises:\n\n1. **Unit-struct emission** — `properties: {}` with empty\n   `required: []` plus `x-rust-struct.derive: []` produces a bare\n   `pub struct StrategyFactory;` declaration. Same shape as\n   `AutoApproveHandler`.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  StrategyFactory:\n    type: object\n    required: []\n    properties: {}\n    description: |\n      Factory for creating import strategy instances. Unit struct;\n      behaviour lives on a hand-written impl block with `create`\n      and `auto_detect` static methods.\n    x-rust-struct:\n      derive: []\n      unit: true\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/fillback/factory.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - StrategyFactory\n    description: |\n      Codegen replaces the unit struct declaration only.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n\n**Verdict:** approved\n\n- [overview] Correctly identifies the unit struct, namespace role, and hand-written impl boundary.\n- [schema] Definition is well-formed: `properties: {}` + `required: []` + `x-rust-struct.derive: []` + `unit: true` matches the AutoApproveHandler precedent.\n- [changes] Two entries cleanly split codegen vs hand-written. `replaces` lists the single struct name; hand-written entry covers all imports and the impl block.\n"
