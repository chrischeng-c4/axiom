"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/ui/mod.md`.

Migrated by batch `projection-core-interfaces-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-ui-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/ui/mod.md"
__legacy_projection_digest__ = "sha256:20d8b4f92282c199cff4b023407b86bdd7570fbd22b367bf786ad63bf38ecd62"


def render_markdown() -> Annotated[str, "sha256:20d8b4f92282c199cff4b023407b86bdd7570fbd22b367bf786ad63bf38ecd62"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-ui-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: core-concept-model-and-invariants\n    claim: core-concept-model-and-invariants\n    coverage: full\n    rationale: \"Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure.\"\n---\n\n# Standardized apps/agentic-workflow/src/ui/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/ui/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `colors` | apps/agentic-workflow/src/ui/mod.rs | module | pub | 3 |  |\n| `progress` | apps/agentic-workflow/src/ui/mod.rs | module | pub | 4 |  |\n| `tables` | apps/agentic-workflow/src/ui/mod.rs | module | pub | 5 |  |\n## Source\n<!-- type: source lang: rust -->\n\n```rust\npub mod colors;\npub mod progress;\npub mod tables;\n\npub use colors::ColorScheme;\npub use progress::ProgressBar;\npub use tables::Table;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/ui/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Regenerate the module declarations and public re-exports directly from\n      the source section.\n```\n"
