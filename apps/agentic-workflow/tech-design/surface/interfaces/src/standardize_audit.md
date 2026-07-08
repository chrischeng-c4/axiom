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

# Standardized apps/agentic-workflow/src/cli/standardize_audit.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/cli/standardize_audit.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ModernizationRisk` | apps/agentic-workflow/src/cli/standardize_audit.rs | enum | pub | 35 |  |
| `PreservationAudit` | apps/agentic-workflow/src/cli/standardize_audit.rs | struct | pub | 50 |  |
| `PreservationSurface` | apps/agentic-workflow/src/cli/standardize_audit.rs | struct | pub | 26 |  |
| `PreservationSurfaceKind` | apps/agentic-workflow/src/cli/standardize_audit.rs | enum | pub | 13 |  |
| `SafeModernizationLever` | apps/agentic-workflow/src/cli/standardize_audit.rs | struct | pub | 43 |  |
| `StandardizeAuditDecision` | apps/agentic-workflow/src/cli/standardize_audit.rs | struct | pub | 60 |  |
| `audit_path` | apps/agentic-workflow/src/cli/standardize_audit.rs | function | pub | 67 | audit_path(project_root: &Path, project: &str) -> PathBuf |
| `evaluate_audit_decision` | apps/agentic-workflow/src/cli/standardize_audit.rs | function | pub | 74 | evaluate_audit_decision(     project_root: &Path,     project: &str,     scopes: &[String],     action_kind: StandardizeActionKind, ) -> StandardizeAuditDecision |
| `fixture_audit` | apps/agentic-workflow/src/cli/standardize_audit.rs | function | pub | 91 | fixture_audit(project: &str, scopes: &[String]) -> PreservationAudit |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=apps/agentic-workflow/src/cli/standardize_audit.rs -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/cli/standardize_audit.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Whole-file source replay owns the preservation audit helper until a
      narrower structural generator can produce it directly.
```
