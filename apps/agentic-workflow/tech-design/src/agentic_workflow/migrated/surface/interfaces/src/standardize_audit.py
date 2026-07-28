"""Canonical Python producer for `apps/agentic-workflow/tech-design/surface/interfaces/src/standardize_audit.md`.

Migrated by batch `projection-surface-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:surface-interfaces/surface-interfaces-src-standardize-audit"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/surface/interfaces/src/standardize_audit.md"
__legacy_projection_digest__ = "sha256:872c16016282191c1178b2769f0ba6cb44cd51d8553e11d225d99e07aeb6ce66"


def render_markdown() -> Annotated[str, "sha256:872c16016282191c1178b2769f0ba6cb44cd51d8553e11d225d99e07aeb6ce66"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-agentic-workflow-src-cli-standardize-audit-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: existing-project-standardization\n    role: primary\n    gap: traceability-closure-gate\n    claim: traceability-closure-gate\n    coverage: full\n    rationale: \"Validation, migration, fillback, and alignment CLI surfaces support standardization and traceability gates.\"\n---\n\n# Standardized apps/agentic-workflow/src/cli/standardize_audit.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/cli/standardize_audit.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `ModernizationRisk` | apps/agentic-workflow/src/cli/standardize_audit.rs | enum | pub | 35 |  |\n| `PreservationAudit` | apps/agentic-workflow/src/cli/standardize_audit.rs | struct | pub | 50 |  |\n| `PreservationSurface` | apps/agentic-workflow/src/cli/standardize_audit.rs | struct | pub | 26 |  |\n| `PreservationSurfaceKind` | apps/agentic-workflow/src/cli/standardize_audit.rs | enum | pub | 13 |  |\n| `SafeModernizationLever` | apps/agentic-workflow/src/cli/standardize_audit.rs | struct | pub | 43 |  |\n| `StandardizeAuditDecision` | apps/agentic-workflow/src/cli/standardize_audit.rs | struct | pub | 60 |  |\n| `audit_path` | apps/agentic-workflow/src/cli/standardize_audit.rs | function | pub | 67 | audit_path(project_root: &Path, project: &str) -> PathBuf |\n| `evaluate_audit_decision` | apps/agentic-workflow/src/cli/standardize_audit.rs | function | pub | 74 | evaluate_audit_decision(     project_root: &Path,     project: &str,     scopes: &[String],     action_kind: StandardizeActionKind, ) -> StandardizeAuditDecision |\n| `fixture_audit` | apps/agentic-workflow/src/cli/standardize_audit.rs | function | pub | 91 | fixture_audit(project: &str, scopes: &[String]) -> PreservationAudit |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-managed-markers -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/cli/standardize_audit.rs -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/cli/standardize_audit.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Whole-file source replay owns the preservation audit helper until a\n      narrower structural generator can produce it directly.\n```\n"
