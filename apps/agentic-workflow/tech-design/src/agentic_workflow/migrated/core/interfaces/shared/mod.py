"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/shared/mod.md`.

Migrated by batch `projection-core-interfaces-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-shared-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/shared/mod.md"
__legacy_projection_digest__ = "sha256:126cedb5ed609a9b6cab8a7ad4bc42d37c2e52752ed3ae894b337ef198c8cbdc"


def render_markdown() -> Annotated[str, "sha256:126cedb5ed609a9b6cab8a7ad4bc42d37c2e52752ed3ae894b337ef198c8cbdc"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-shared-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: core-concept-model-and-invariants\n    claim: core-concept-model-and-invariants\n    coverage: full\n    rationale: \"Shared workflow utilities support the single AW CLI across lifecycle phases.\"\n---\n\n# Standardized apps/agentic-workflow/src/shared/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/shared/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `services` | apps/agentic-workflow/src/shared/mod.rs | module | pub | 10 |  |\n| `tools` | apps/agentic-workflow/src/shared/mod.rs | module | pub | 11 |  |\n| `workspace` | apps/agentic-workflow/src/shared/mod.rs | module | pub | 12 |  |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-handwrite -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/shared/mod.rs -->\n```rust\n//! Shared utilities and services used across workflows\n//!\n//! File and spec services, plus tool integration points used by workflow\n//! phases. The legacy `cli` re-export submodule was deleted during the\n//! Score unbundling — all user-facing CLI commands now live in\n//! `apps/agentic-workflow/`.\n\npub mod services;\npub mod tools;\npub mod workspace;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/shared/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Source template owns the complete shared module facade.\n```\n"
