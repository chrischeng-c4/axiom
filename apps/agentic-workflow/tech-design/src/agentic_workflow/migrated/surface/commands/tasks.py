"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/surface/commands/tasks.md`.

Migrated by batch `semantic-surface-commands-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:surface-commands/surface-commands-tasks"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/surface/commands/tasks.md"
__legacy_td_digest__ = "sha256:4dbbc0528d2fb54647aac2d43cfbbef0b266761697626032a4e64143bb7257c3"


def render_markdown() -> Annotated[str, "sha256:4dbbc0528d2fb54647aac2d43cfbbef0b266761697626032a4e64143bb7257c3"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: score-tasks\nfill_sections: [schema, changes]\ncapability_refs:\n  - id: workflow-root-runner\n    role: primary\n    gap: cli-workflow-chain\n    claim: cli-workflow-chain\n    coverage: full\n    rationale: \"Command/root TDs support CLI workflow chain routing and root-runner dispatch.\"\n---\n\n# TasksCommands Type\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  TasksCommands:\n    type: string\n    enum: [Generate, Create]\n    description: Available subcommands for `score tasks`.\n    x-rust-enum:\n      derive: [Subcommand]\n      variants:\n        - name: Generate\n          kind: struct\n          doc: \"Auto-generate tasks from specs (recommended).\"\n          fields:\n            - { name: change_id, rust_type: String }\n        - name: Create\n          kind: struct\n          doc: \"Create tasks file from JSON file (legacy, for manual override).\"\n          fields:\n            - { name: change_id, rust_type: String }\n            - { name: json_file, rust_type: PathBuf, \"x-clap-arg\": \"long\" }\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/cli/tasks.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - TasksCommands\n    description: |\n      Codegen replaces the enum declaration only.\n  - path: apps/agentic-workflow/src/cli/tasks.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: |\n      Hand-written outside CODEGEN: module docstring, imports, the\n      `run` function and JSON parsing logic.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: review lang: markdown -->\n\n**Verdict:** approved\n\n- [overview] Subcommand enum with struct variants, one with variant-field clap-arg.\n- [schema] x-clap-arg on json_file field of Create variant.\n- [changes] Standard split.\n"
