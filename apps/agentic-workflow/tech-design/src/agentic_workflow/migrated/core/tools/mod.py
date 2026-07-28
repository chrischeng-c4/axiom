"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/tools/mod.md`.

Migrated by batch `semantic-core-tools-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-mod"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/tools/mod.md"
__legacy_td_digest__ = "sha256:8b5a7659646dc10a13908a76913015966340af0d0ead8e824f8feadfbb53396e"


def render_markdown() -> Annotated[str, "sha256:8b5a7659646dc10a13908a76913015966340af0d0ead8e824f8feadfbb53396e"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-tools-mod\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands.\"\n---\n\n# Tool Registry Types\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nMCP tool registry types in `apps/agentic-workflow/src/tools/mod.rs`. Two shapes:\n\n- `ToolRegistry` — `tools: Vec<ToolDefinition>` (private). No derives.\n- `ToolDefinition` — `name`, `description`, `input_schema: Value`.\n  Derives `[Clone]`.\n\nCodegen replaces both type declarations. Module submodule\ndeclarations, imports, the `impl ToolRegistry` block stay\nhand-written.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  ToolRegistry:\n    type: object\n    required: [tools]\n    description: Registry of available MCP tools.\n    properties:\n      tools:\n        type: array\n        items: { type: object }\n        x-rust-type: \"Vec<ToolDefinition>\"\n        x-rust-visibility: private\n        description: \"Registered tools.\"\n    x-rust-struct:\n      derive: []\n\n  ToolDefinition:\n    type: object\n    required: [name, description, input_schema]\n    description: Tool definition for MCP protocol.\n    properties:\n      name:\n        type: string\n        description: \"Tool name.\"\n      description:\n        type: string\n        description: \"Human-readable description.\"\n      input_schema:\n        type: object\n        x-rust-type: \"Value\"\n        description: \"JSON Schema for tool input.\"\n    x-rust-struct:\n      derive: [Clone]\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/tools/mod.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - ToolRegistry\n      - ToolDefinition\n    description: |\n      Codegen replaces both type declarations.\n  - path: apps/agentic-workflow/src/tools/mod.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: |\n      Hand-written outside CODEGEN: module submodule declarations,\n      `pub use` re-exports, imports, the `impl ToolRegistry` block.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: review lang: markdown -->\n**Verdict:** approved\n\n- [overview] 2 structs; standard pattern.\n- [schema] Both well-formed.\n- [changes] Standard split.\n"
