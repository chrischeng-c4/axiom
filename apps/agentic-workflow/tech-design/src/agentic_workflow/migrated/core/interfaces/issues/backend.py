"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/interfaces/issues/backend.md`.

Migrated by batch `semantic-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-issues-backend"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/interfaces/issues/backend.md"
__legacy_td_digest__ = "sha256:9f700d9583bde960aee01b9aafaaac72f4966209cb7791b930824f17d362a498"


def render_markdown() -> Annotated[str, "sha256:9f700d9583bde960aee01b9aafaaac72f4966209cb7791b930824f17d362a498"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-issues-backend\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: agent-first-cli-product-model\n    claim: agent-first-cli-product-model\n    coverage: full\n    rationale: \"Issue backend interfaces project the single AW CLI workflow state to configured issue platforms.\"\n---\n\n# SyncReport Type\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSummary type for the issue sync function in\n`apps/agentic-workflow/src/issues/backend.rs`. One shape:\n\n- `SyncReport` — `fetched: usize`, `created: usize`, `updated: usize`.\n  Derives `[Debug, Clone, Copy]`. Pure data carrier returned by\n  `pub async fn sync_issues(...)`.\n\nCodegen replaces only the SyncReport struct declaration. The\n`IssueBackend` trait declaration with all its async methods, the\n`sync_issues` function, and module imports stay hand-written.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  SyncReport:\n    type: object\n    required: [fetched, created, updated]\n    description: |\n      Summary of an issue sync operation.\n    properties:\n      fetched:\n        type: integer\n        x-rust-type: \"usize\"\n        description: \"Number of issues fetched from source.\"\n      created:\n        type: integer\n        x-rust-type: \"usize\"\n        description: \"Number of issues newly created on target.\"\n      updated:\n        type: integer\n        x-rust-type: \"usize\"\n        description: \"Number of issues updated on target.\"\n    x-rust-struct:\n      derive: [Debug, Clone, Copy]\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/issues/backend.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - SyncReport\n    description: |\n      Codegen replaces the SyncReport struct declaration only.\n  - path: apps/agentic-workflow/src/issues/backend.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: |\n      Hand-written outside CODEGEN: module docstring, imports, the\n      `IssueBackend` trait declaration with all its async methods,\n      and the `sync_issues` function.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n**Verdict:** approved\n\n- [overview] Pure data carrier with three usize fields and Copy.\n- [schema] Standard usize via x-rust-type pattern.\n- [changes] Hand-written boundary correctly preserves the trait declaration.\n"
