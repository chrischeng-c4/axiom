"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/issues/backends/mod.md`.

Migrated by batch `projection-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-issues-backends-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/issues/backends/mod.md"
__legacy_projection_digest__ = "sha256:560e4af535a640c1eff46c4dcb9dec72c763d5797c26fc4072d5f8c1685ba930"


def render_markdown() -> Annotated[str, "sha256:560e4af535a640c1eff46c4dcb9dec72c763d5797c26fc4072d5f8c1685ba930"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-issues-backends-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: agent-first-cli-product-model\n    claim: agent-first-cli-product-model\n    coverage: full\n    rationale: \"Issue backend interfaces project the single AW CLI workflow state to configured issue platforms.\"\n---\n\n# Standardized apps/agentic-workflow/src/issues/backends/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/issues/backends/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `github` | apps/agentic-workflow/src/issues/backends/mod.rs | module | pub | 5 |  |\n| `gitlab` | apps/agentic-workflow/src/issues/backends/mod.rs | module | pub | 6 |  |\n| `local` | apps/agentic-workflow/src/issues/backends/mod.rs | module | pub | 7 |  |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap standardize:claim-code -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/issues/backends/mod.rs -->\n```rust\n//! Issue storage backend implementations.\n\npub mod github;\npub mod gitlab;\npub mod local;\n\npub use github::GitHubBackend;\npub use gitlab::GitLabBackend;\npub use local::LocalBackend;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/issues/backends/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:standardize:claim-code>\"\n    description: |\n      Source template owns the issue backend facade module.\n```\n"
