---
id: projects-agentic-workflow-src-cli-standardize-audit-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: traceability-closure-gate
    claim: traceability-closure-gate
    coverage: full
    rationale: "Validation, migration, fillback, and alignment CLI surfaces support standardization and traceability gates."
---

# Standardized projects/agentic-workflow/src/cli/standardize_audit.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/cli/standardize_audit.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ModernizationRisk` | projects/agentic-workflow/src/cli/standardize_audit.rs | enum | pub | 35 |  |
| `PreservationAudit` | projects/agentic-workflow/src/cli/standardize_audit.rs | struct | pub | 50 |  |
| `PreservationSurface` | projects/agentic-workflow/src/cli/standardize_audit.rs | struct | pub | 26 |  |
| `PreservationSurfaceKind` | projects/agentic-workflow/src/cli/standardize_audit.rs | enum | pub | 13 |  |
| `SafeModernizationLever` | projects/agentic-workflow/src/cli/standardize_audit.rs | struct | pub | 43 |  |
| `StandardizeAuditDecision` | projects/agentic-workflow/src/cli/standardize_audit.rs | struct | pub | 60 |  |
| `audit_path` | projects/agentic-workflow/src/cli/standardize_audit.rs | function | pub | 67 | audit_path(project_root: &Path, project: &str) -> PathBuf |
| `evaluate_audit_decision` | projects/agentic-workflow/src/cli/standardize_audit.rs | function | pub | 74 | evaluate_audit_decision(     project_root: &Path,     project: &str,     scopes: &[String],     action_kind: StandardizeActionKind, ) -> StandardizeAuditDecision |
| `fixture_audit` | projects/agentic-workflow/src/cli/standardize_audit.rs | function | pub | 91 | fixture_audit(project: &str, scopes: &[String]) -> PreservationAudit |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/cli/standardize_audit.rs -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/standardize_audit.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Whole-file source replay owns the preservation audit helper until a
      narrower structural generator can produce it directly.
```
