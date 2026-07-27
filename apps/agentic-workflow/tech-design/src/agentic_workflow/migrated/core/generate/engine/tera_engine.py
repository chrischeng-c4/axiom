"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/engine/tera_engine.md`.

Migrated by batch `semantic-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-engine-tera-engine"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/engine/tera_engine.md"
__legacy_td_digest__ = "sha256:80fc91c134b3336efe6bb9972f7ab052affb0c71c0771e0b79af19cd8178b3cd"


def render_markdown() -> Annotated[str, "sha256:80fc91c134b3336efe6bb9972f7ab052affb0c71c0771e0b79af19cd8178b3cd"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-generate-engine-tera-engine\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# TemplateEngine Type\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nTera template engine wrapper in\n`apps/agentic-workflow/src/generate/engine/tera_engine.rs`. One shape:\n\n- `TemplateEngine` — single private `tera: Tera` field with no\n  derives. Wraps the `tera::Tera` engine to provide template\n  loading, registration of custom filters, and rendering. All\n  behaviour lives on the source-template-owned `impl TemplateEngine` block\n  (`new`, `empty`, `add_template`, `render`, etc.).\n\nCodegen replaces the struct declaration. Companion source templates own the\nmodule preamble and runtime implementation blocks that previously lived in\nmanaged HANDWRITE gaps.\n\nThis spec exercises:\n\n1. **No-derive struct emission** — `x-rust-struct.derive: []` emits\n   `pub struct TemplateEngine { ... }` with no `#[derive(...)]`.\n2. **`x-rust-visibility: private`** on the only field — keeps\n   `tera: Tera` private (no `pub`) on a public struct.\n3. **`x-rust-type: \"Tera\"`** in `required:` — uses the bare\n   foreign type without Option auto-wrapping.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  TemplateEngine:\n    type: object\n    required: [tera]\n    description: |\n      Tera template engine wrapper. Holds the `tera::Tera` engine\n      with custom filters registered. All behaviour is on the\n      hand-written impl block.\n    properties:\n      tera:\n        type: string\n        x-rust-type: \"Tera\"\n        x-rust-visibility: private\n        description: \"Underlying tera engine.\"\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/engine/tera_engine.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - TemplateEngine\n    description: |\n      Codegen replaces the struct declaration only.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n**Verdict:** approved\n\n- [overview] Correctly identifies the single struct, its private foreign-type field, lack of derives, and hand-written boundary.\n- [schema] Definition is well-formed: `x-rust-struct.derive: []`, `tera` in `required:` with `x-rust-type: \"Tera\"` to use the bare foreign type, and `x-rust-visibility: private` to keep field non-`pub`.\n- [changes] Two entries cleanly split codegen vs hand-written. `replaces` lists the single struct name; hand-written entry covers module-level items and the entire impl block.\n"
