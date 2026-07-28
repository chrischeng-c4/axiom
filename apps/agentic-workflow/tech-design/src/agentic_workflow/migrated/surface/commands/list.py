"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/surface/commands/list.md`.

Migrated by batch `semantic-surface-commands-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:surface-commands/surface-commands-list"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/surface/commands/list.md"
__legacy_td_digest__ = "sha256:616a90940a2ca671394b0a030ccaba16c6ee96f54c2b38687f209e04ed215705"


def render_markdown() -> Annotated[str, "sha256:616a90940a2ca671394b0a030ccaba16c6ee96f54c2b38687f209e04ed215705"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: score-list\nfill_sections: [schema, changes]\ncapability_refs:\n  - id: workflow-root-runner\n    role: primary\n    gap: cli-workflow-chain\n    claim: cli-workflow-chain\n    coverage: full\n    rationale: \"Command/root TDs support CLI workflow chain routing and root-runner dispatch.\"\n---\n\n# List Output Types\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  ActiveItem:\n    type: object\n    required: [slug, phase, branch, title]\n    description: |\n      Active change item: a change with a worktree and known phase.\n    properties:\n      slug:\n        type: string\n        description: \"Change slug.\"\n      phase:\n        type: string\n        description: \"Current phase from the nested issue file. \\\"unknown\\\" when unreadable.\"\n      branch:\n        type: string\n        description: \"Branch name. Always cclab/<slug>.\"\n      title:\n        type: string\n        description: \"Issue title.\"\n    x-rust-struct:\n      derive: [Serialize, Debug, Clone, PartialEq]\n\n  IdleItem:\n    type: object\n    required: [slug, state, title]\n    description: |\n      Idle issue item: an issue without a matching worktree.\n    properties:\n      slug:\n        type: string\n        description: \"Issue slug.\"\n      state:\n        type: string\n        description: \"Issue state. \\\"open\\\" or \\\"draft\\\".\"\n      title:\n        type: string\n        description: \"Issue title.\"\n    x-rust-struct:\n      derive: [Serialize, Debug, Clone, PartialEq]\n\n  ListOutput:\n    type: object\n    required: [active, idle]\n    description: |\n      Aggregated output from `score list`.\n    properties:\n      active:\n        type: array\n        items: { type: object }\n        x-rust-type: \"Vec<ActiveItem>\"\n        description: \"Active items.\"\n      idle:\n        type: array\n        items: { type: object }\n        x-rust-type: \"Vec<IdleItem>\"\n        description: \"Idle items.\"\n    x-rust-struct:\n      derive: [Serialize, Debug, Default]\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/cli/list.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - ActiveItem\n      - IdleItem\n      - ListOutput\n    description: |\n      Codegen replaces all three struct declarations.\n  - path: apps/agentic-workflow/src/cli/list.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: |\n      Hand-written outside CODEGEN: module docstring, imports, the\n      `run_dual_source` function and all helpers.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: review lang: markdown -->\n\n**Verdict:** approved\n\n- [overview] Three pure data carriers; standard Serialize derive.\n- [schema] All well-formed; Vec<T> via x-rust-type for ListOutput fields.\n- [changes] Standard split with all three structs in `replaces`.\n"
