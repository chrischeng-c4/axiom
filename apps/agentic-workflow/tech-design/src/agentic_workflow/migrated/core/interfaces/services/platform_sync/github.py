"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/interfaces/services/platform_sync/github.md`.

Migrated by batch `semantic-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-services-platform-sync-github"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/interfaces/services/platform_sync/github.md"
__legacy_td_digest__ = "sha256:e4ae61383c5afe4f73e718d57f73f246733fd95e39636f16f96be9e787a3170c"


def render_markdown() -> Annotated[str, "sha256:e4ae61383c5afe4f73e718d57f73f246733fd95e39636f16f96be9e787a3170c"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-services-platform-sync-github\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: agent-first-cli-product-model\n    claim: agent-first-cli-product-model\n    coverage: full\n    rationale: \"Service interfaces expose project, issue, and platform behavior to the single AW CLI.\"\n---\n\n# GitHubProvider Type\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nGitHub platform-sync provider in\n`apps/agentic-workflow/src/services/platform_sync/github.rs`. One shape:\n\n- `GitHubProvider` — `config: PlatformConfig`, `token: Option<String>`,\n  no derives. Both fields private.\n\nCodegen replaces the struct declaration. Companion source templates own module\ndocumentation, imports, provider construction, auth checks, sync orchestration,\nAPI and CLI upserts, error sanitization, URL parsing, and regression tests.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  GitHubProvider:\n    type: object\n    required: [config, token]\n    description: |\n      GitHub platform-sync provider.\n    properties:\n      config:\n        type: object\n        x-rust-type: \"PlatformConfig\"\n        x-rust-visibility: private\n        description: \"Platform configuration.\"\n      token:\n        type: string\n        x-rust-type: \"Option<String>\"\n        x-rust-visibility: private\n        description: \"Optional auth token.\"\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/services/platform_sync/github.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - GitHubProvider\n    description: |\n      Codegen replaces the struct declaration only.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n**Verdict:** approved\n\n- [overview] Single struct with no derives + 2 private fields.\n- [schema] Both fields via x-rust-type with private visibility.\n- [changes] Standard split.\n"
