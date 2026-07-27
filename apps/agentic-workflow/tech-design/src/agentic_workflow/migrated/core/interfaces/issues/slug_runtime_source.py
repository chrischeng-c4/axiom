"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/issues/slug_runtime_source.md`.

Migrated by batch `projection-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-issues-slug-runtime-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/issues/slug_runtime_source.md"
__legacy_projection_digest__ = "sha256:c4152064c26aca1338eb9a5c93ea563066679f91419462a6fcf9a944deb9544a"


def render_markdown() -> Annotated[str, "sha256:c4152064c26aca1338eb9a5c93ea563066679f91419462a6fcf9a944deb9544a"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-issues-slug-runtime-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: agent-first-cli-product-model\n    claim: agent-first-cli-product-model\n    coverage: full\n    rationale: \"Issue backend interfaces project the single AW CLI workflow state to configured issue platforms.\"\n---\n\n# Issue Slug Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/issues/slug.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `BranchKind` | apps/agentic-workflow/src/issues/slug.rs | enum | pub | 28 |  |\n| `ResolvedId` | apps/agentic-workflow/src/issues/slug.rs | enum | pub | 48 |  |\n| `SlugAliases` | apps/agentic-workflow/src/issues/slug.rs | struct | pub | 77 |  |\n| `as_prefix` | apps/agentic-workflow/src/issues/slug.rs | function | pub | 36 | as_prefix(&self) -> &'static str |\n| `build_branch_name` | apps/agentic-workflow/src/issues/slug.rs | function | pub | 201 | build_branch_name(kind: BranchKind, id: u64, title: &str) -> String |\n| `build_canonical_slug` | apps/agentic-workflow/src/issues/slug.rs | function | pub | 120 | build_canonical_slug(id: u64, _title: &str) -> String |\n| `id` | apps/agentic-workflow/src/issues/slug.rs | function | pub | 59 | id(&self) -> u64 |\n| `insert` | apps/agentic-workflow/src/issues/slug.rs | function | pub | 112 | insert(&mut self, legacy_slug: String, id: u64) |\n| `is_legacy` | apps/agentic-workflow/src/issues/slug.rs | function | pub | 67 | is_legacy(&self) -> bool |\n| `load` | apps/agentic-workflow/src/issues/slug.rs | function | pub | 85 | load(project_root: &Path) -> Result<Self> |\n| `lookup` | apps/agentic-workflow/src/issues/slug.rs | function | pub | 108 | lookup(&self, legacy_slug: &str) -> Option<u64> |\n| `parse_branch_name` | apps/agentic-workflow/src/issues/slug.rs | function | pub | 180 | parse_branch_name(branch: &str) -> Option<(BranchKind, u64)> |\n| `parse_slug_input` | apps/agentic-workflow/src/issues/slug.rs | function | pub | 132 | parse_slug_input(input: &str, aliases: &SlugAliases) -> Result<ResolvedId> |\n| `save` | apps/agentic-workflow/src/issues/slug.rs | function | pub | 98 | save(&self, project_root: &Path) -> Result<()> |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap missing-generator:slug-parser-builder-primitives -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/issues/slug.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:missing-generator:slug-parser-builder-primitives>\"\n    description: \"Source template owns slug parser, alias, and branch primitives.\"\n```\n"
