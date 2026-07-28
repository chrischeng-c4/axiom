"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/generators/mod.md`.

Migrated by batch `semantic-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-mod"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/mod.md"
__legacy_td_digest__ = "sha256:64639e78657fa59a00d2f6a82dbc5a9b775dd645cc11702e707c6b3b1aa694a7"


def render_markdown() -> Annotated[str, "sha256:64639e78657fa59a00d2f6a82dbc5a9b775dd645cc11702e707c6b3b1aa694a7"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-generate-generators-mod\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# GeneratorArgs Type\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nGenerator arguments type in `apps/agentic-workflow/src/generators/mod.rs`. One shape:\n\n- `GeneratorArgs` — `section_type: SectionType`,\n  `sdd_id: Option<String>`, `sdd_refs: Vec<String>`. Derives `[Debug, Clone]`.\n\nCodegen replaces the struct declaration. Companion source templates own the\nmodule preamble/imports and the runtime helper/dispatch/test block that\npreviously lived in managed HANDWRITE gaps.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  GeneratorArgs:\n    type: object\n    required: [section_type, sdd_id, sdd_refs]\n    description: Arguments for invoking a structural generator.\n    properties:\n      section_type:\n        type: string\n        x-rust-type: \"SectionType\"\n        description: \"Target section type.\"\n      sdd_id:\n        type: string\n        x-rust-type: \"Option<String>\"\n        description: \"Change ID providing context (from --sdd-id).\"\n      sdd_refs:\n        type: array\n        items: { type: string }\n        x-rust-type: \"Vec<String>\"\n        description: \"Related spec references (from --sdd-refs).\"\n    x-rust-struct:\n      derive: [Debug, Clone]\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generators/mod.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - GeneratorArgs\n    description: |\n      Codegen replaces the struct declaration only.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n\n**Verdict:** approved\n\n- [overview] Single struct with foreign-type fields.\n- [schema] Standard pattern.\n- [changes] Standard split.\n"
