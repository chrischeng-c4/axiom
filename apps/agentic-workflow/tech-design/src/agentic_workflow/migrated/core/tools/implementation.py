"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/tools/implementation.md`.

Migrated by batch `semantic-core-tools-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-implementation"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/tools/implementation.md"
__legacy_td_digest__ = "sha256:a19048180cd6f9b710dba12c6847281155a0c9e09f291abcdae01f2afa978ff3"


def render_markdown() -> Annotated[str, "sha256:a19048180cd6f9b710dba12c6847281155a0c9e09f291abcdae01f2afa978ff3"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: projects-sdd-src-tools-implementation-rs\nfill_sections: [overview, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands.\"\n---\n\n# Standardized apps/agentic-workflow/src/tools/implementation.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nImplementation support MCP tools are managed by source fragments under\n`apps/agentic-workflow/tech-design/core/tools/implementation/`. The split keeps each\nfragment below the spec hard size limit while exercising module preamble,\nmodule trailer, symbol, and test-module replacement.\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/tools/implementation.rs\n    section: source\n    action: modify\n    impl_mode: codegen\n    description: |\n      Whole-file HANDWRITE wrapper is removed through source-fragment\n      composition. See implementation/*.md for the concrete generated regions.\n```\n"
