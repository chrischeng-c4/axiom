"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-mod-rs.md`.

Migrated by batch `projection-core-validate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-validate/core-validate-source-projects-sdd-src-validate-mod-rs"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-mod-rs.md"
__legacy_projection_digest__ = "sha256:898385e3fb31300db8f05bb510c872d7db37099cf199b8e7fe0833f893064c64"


def render_markdown() -> Annotated[str, "sha256:898385e3fb31300db8f05bb510c872d7db37099cf199b8e7fe0833f893064c64"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-validate-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: existing-project-standardization\n    role: primary\n    gap: managed-and-semantic-production-gates\n    claim: managed-and-semantic-production-gates\n    coverage: full\n    rationale: \"Validation TDs implement managed and semantic production gates for standardization readiness.\"\n---\n\n# Standardized apps/agentic-workflow/src/validate/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/validate/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `router` | apps/agentic-workflow/src/validate/mod.rs | module | pub | 23 |  |\n| `rule` | apps/agentic-workflow/src/validate/mod.rs | module | pub | 24 |  |\n| `rules` | apps/agentic-workflow/src/validate/mod.rs | module | pub | 25 |  |\n| `runner` | apps/agentic-workflow/src/validate/mod.rs | module | pub | 26 |  |\n## Source\n<!-- type: source lang: rust -->\n\n````rust\n//! TD spec rule checker.\n//!\n//! Runs authoring-lint + consistency rules against tech-design specs.\n//! Called by `aw td validate <path>` where `<path>` is a slug (commit-gate),\n//! a spec-space directory prefix (read-only walk), or a single spec file.\n//!\n//! Distinct from `crate::validator` (which validates generic spec document\n//! structure — headings, scenarios, WHEN/THEN). This module validates\n//! TD-specific content rules: rust_type shape, x-mamba-binding integrity,\n//! impl_mode discipline, cross-section consistency.\n//!\n//! Rule catalog (R-ids from issue `enhancement-split-validate-spec-side-from-audit-code-side-cove`):\n//! - R3a: double-Option — reject `Option<Option<T>>` in any `rust_type`\n//! - R3b: nullable/required contradiction\n//! - R3c: orphan x-mamba-binding\n//! - R3d: lowercase enum rust_type\n//! - R3e: impl_mode misuse\n//! - R3f: codegen-ready gate (Mermaid Plus frontmatter; skipped for Rule 2-2)\n//! - R3g: cross-section rust_type consistency\n\npub mod router;\npub mod rule;\npub mod rules;\npub mod runner;\n\npub use router::{classify, resolve_spec_files, PathShape};\npub use rule::{Finding, Rule, RuleId, RuleReport, Severity};\npub use runner::run_rules;\n````\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/validate/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Regenerate the remaining validation module source directly from the\n      source section. Existing schema CODEGEN blocks, when present, remain\n      owned by their semantic specs.\n```\n"
