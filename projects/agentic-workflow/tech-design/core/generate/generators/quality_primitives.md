---
id: projects-agentic-workflow-src-generate-generators-quality-primitives-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Standardized projects/agentic-workflow/src/generate/generators/quality_primitives.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/generators/quality_primitives.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `PrimitiveDialCompatibility` | projects/agentic-workflow/src/generate/generators/quality_primitives.rs | struct | pub | 42 |  |
| `PrimitiveDialSupport` | projects/agentic-workflow/src/generate/generators/quality_primitives.rs | enum | pub | 11 |  |
| `PrimitiveEvidenceExample` | projects/agentic-workflow/src/generate/generators/quality_primitives.rs | struct | pub | 60 |  |
| `PrimitiveEvidenceKind` | projects/agentic-workflow/src/generate/generators/quality_primitives.rs | enum | pub | 30 |  |
| `PrimitiveReviewCheck` | projects/agentic-workflow/src/generate/generators/quality_primitives.rs | struct | pub | 51 |  |
| `PrimitiveReviewFinding` | projects/agentic-workflow/src/generate/generators/quality_primitives.rs | struct | pub | 104 |  |
| `PrimitiveReviewSeverity` | projects/agentic-workflow/src/generate/generators/quality_primitives.rs | enum | pub | 21 |  |
| `PrimitiveSelectionCitation` | projects/agentic-workflow/src/generate/generators/quality_primitives.rs | struct | pub | 93 |  |
| `PrimitiveSelectionRequest` | projects/agentic-workflow/src/generate/generators/quality_primitives.rs | struct | pub | 84 |  |
| `QualityPrimitiveProfile` | projects/agentic-workflow/src/generate/generators/quality_primitives.rs | struct | pub | 68 |  |
| `default_quality_primitive_profiles` | projects/agentic-workflow/src/generate/generators/quality_primitives.rs | function | pub | 112 | default_quality_primitive_profiles() -> Vec<QualityPrimitiveProfile> |
| `evaluate_primitive_review_checks` | projects/agentic-workflow/src/generate/generators/quality_primitives.rs | function | pub | 313 | evaluate_primitive_review_checks(     profile: &QualityPrimitiveProfile,     artifact_text: &str, ) -> Vec<PrimitiveReviewFinding> |
| `explain_primitive_selection` | projects/agentic-workflow/src/generate/generators/quality_primitives.rs | function | pub | 259 | explain_primitive_selection(     profiles: &[QualityPrimitiveProfile],     request: &PrimitiveSelectionRequest, ) -> PrimitiveSelectionCitation |
| `find_quality_primitive_profile` | projects/agentic-workflow/src/generate/generators/quality_primitives.rs | function | pub | 192 | find_quality_primitive_profile(name: &str) -> Option<QualityPrimitiveProfile> |
| `validate_quality_primitive_profiles` | projects/agentic-workflow/src/generate/generators/quality_primitives.rs | function | pub | 200 | validate_quality_primitive_profiles(profiles: &[QualityPrimitiveProfile]) -> Vec<String> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/generators/quality_primitives.rs -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/generators/quality_primitives.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Whole-file source replay owns the quality primitive profile registry until
      the generator can derive these profiles from typed primitive metadata.
```
