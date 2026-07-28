"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/interfaces/issues/backends/gitlab.md`.

Migrated by batch `semantic-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-issues-backends-gitlab"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/interfaces/issues/backends/gitlab.md"
__legacy_td_digest__ = "sha256:cd45bdd214a361f90a7a0e6bc5b4c4b21b4809076e2084d9825ab08605639b70"


def render_markdown() -> Annotated[str, "sha256:cd45bdd214a361f90a7a0e6bc5b4c4b21b4809076e2084d9825ab08605639b70"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-issues-backends-gitlab\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: agent-first-cli-product-model\n    claim: agent-first-cli-product-model\n    coverage: full\n    rationale: \"Issue backend interfaces project the single AW CLI workflow state to configured issue platforms.\"\n---\n\n# GitLabBackend Type\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nIssue backend struct in `apps/agentic-workflow/src/issues/backends/gitlab.rs`.\nOne shape:\n\n- `GitLabBackend` — single private `repo: Option<String>` field with\n  no derives.\n\nCodegen replaces the struct declaration. Companion source templates own module\ndocumentation/imports, GitLab CLI plumbing, `IssueBackend` behavior, parsing,\nsubprocess execution, and regression tests.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  GitLabBackend:\n    type: object\n    required: [repo]\n    description: |\n      Issue backend that calls the `glab` CLI.\n    properties:\n      repo:\n        type: string\n        x-rust-type: \"Option<String>\"\n        x-rust-visibility: private\n        description: \"Optional `owner/repo` slug. None = use CWD-detected repo.\"\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/issues/backends/gitlab.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - GitLabBackend\n    description: |\n      Codegen replaces the struct declaration only.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n**Verdict:** approved\n\n- [overview] Single struct with no derives + private field.\n- [schema] Option<String> via x-rust-type with private visibility.\n- [changes] Standard split.\n"
