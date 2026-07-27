"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/surface/commands/changes.md`.

Migrated by batch `semantic-surface-commands-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:surface-commands/surface-commands-changes"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/surface/commands/changes.md"
__legacy_td_digest__ = "sha256:6e00fbbe71686548f317eb539568d46cbe22c37d9c2931b52b2565e562b18366"


def render_markdown() -> Annotated[str, "sha256:6e00fbbe71686548f317eb539568d46cbe22c37d9c2931b52b2565e562b18366"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: score-changes\nfill_sections: [schema, changes]\ncapability_refs:\n  - id: workflow-root-runner\n    role: primary\n    gap: cli-workflow-chain\n    claim: cli-workflow-chain\n    coverage: full\n    rationale: \"Command/root TDs support CLI workflow chain routing and root-runner dispatch.\"\n---\n\n# ChangesArgs + ChangesCommand Types\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  ChangesArgs:\n    type: object\n    required: [command]\n    description: Args wrapper for `score changes`.\n    properties:\n      command:\n        type: object\n        x-rust-type: \"ChangesCommand\"\n        x-clap-command: \"subcommand\"\n        description: \"The selected subcommand.\"\n    x-rust-struct:\n      derive: [Debug, Args]\n\n  ChangesCommand:\n    type: string\n    enum: [FilesAffected]\n    description: Available subcommands for `score changes`.\n    x-rust-enum:\n      derive: [Debug, Subcommand]\n      variants:\n        - name: FilesAffected\n          kind: struct\n          doc: \"List files that a change will modify (from spec Changes section).\"\n          fields:\n            - { name: change_id, rust_type: String }\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/cli/changes.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - ChangesArgs\n      - ChangesCommand\n    description: |\n      Codegen replaces both type declarations.\n  - path: apps/agentic-workflow/src/cli/changes.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: |\n      Hand-written outside CODEGEN: module docstring, imports, the `run`\n      function, and all helpers + tests.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: review lang: markdown -->\n\n**Verdict:** approved\n\n- [overview] Args wrapper + Subcommand enum, the standard clap pattern.\n- [schema] x-clap-command \"subcommand\" on ChangesArgs.command; struct-variant on the enum side.\n- [changes] Standard split.\n"
