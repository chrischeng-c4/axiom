"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/interfaces/issues/backends/local.md`.

Migrated by batch `semantic-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-issues-backends-local"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/interfaces/issues/backends/local.md"
__legacy_td_digest__ = "sha256:aba797bd36ebc1798f5bdc94b6bacc84f0c8eb31fa919abd53cb2b3b1af2a7f4"


def render_markdown() -> Annotated[str, "sha256:aba797bd36ebc1798f5bdc94b6bacc84f0c8eb31fa919abd53cb2b3b1af2a7f4"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-issues-backends-local\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: agent-first-cli-product-model\n    claim: agent-first-cli-product-model\n    coverage: full\n    rationale: \"Issue backend interfaces project the single AW CLI workflow state to configured issue platforms.\"\n---\n\n# LocalBackend Type\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nLocal filesystem issue backend in `apps/agentic-workflow/src/issues/backends/local.rs`.\nIt can be rooted at the repository lifecycle directory (`.aw/issues`) or at\nan ephemeral remote read cache under `/tmp/aw/issues`. One shape:\n\n- `LocalBackend` — single private `issues_dir: PathBuf` field, no derives.\n\nCodegen replaces the struct declaration. Companion source templates own module\nimports, helper functions, `impl LocalBackend { ... }`, trait impls,\nfrontmatter persistence adapters, and regression tests.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  LocalBackend:\n    type: object\n    required: [issues_dir]\n    description: Backend that stores issues as files under an issue directory.\n    properties:\n      issues_dir:\n        type: string\n        x-rust-type: \"PathBuf\"\n        x-rust-visibility: private\n        description: \"Directory containing issue files.\"\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/issues/backends/local.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - LocalBackend\n    description: |\n      Codegen replaces the struct only.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n**Verdict:** approved\n\n- [overview] Single struct with private PathBuf.\n- [schema] Standard pattern.\n- [changes] Standard split.\n"
