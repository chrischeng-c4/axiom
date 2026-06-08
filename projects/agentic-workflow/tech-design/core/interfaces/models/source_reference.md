---
id: projects-agentic-workflow-src-models-source-reference-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Standardized projects/agentic-workflow/src/models/source_reference.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/models/source_reference.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SourceFailureMode` | projects/agentic-workflow/src/models/source_reference.rs | enum | pub | 28 |  |
| `SourceReference` | projects/agentic-workflow/src/models/source_reference.rs | struct | pub | 54 |  |
| `SourceReferenceAvailability` | projects/agentic-workflow/src/models/source_reference.rs | enum | pub | 19 |  |
| `SourceReferenceKind` | projects/agentic-workflow/src/models/source_reference.rs | enum | pub | 8 |  |
| `SourceReferencePolicy` | projects/agentic-workflow/src/models/source_reference.rs | struct | pub | 46 |  |
| `SourceReferenceRequirement` | projects/agentic-workflow/src/models/source_reference.rs | struct | pub | 36 |  |
| `SourceReferenceReview` | projects/agentic-workflow/src/models/source_reference.rs | struct | pub | 87 |  |
| `SourceReviewFinding` | projects/agentic-workflow/src/models/source_reference.rs | struct | pub | 79 |  |
| `SourceReviewSeverity` | projects/agentic-workflow/src/models/source_reference.rs | enum | pub | 71 |  |
| `evaluate_source_references` | projects/agentic-workflow/src/models/source_reference.rs | function | pub | 94 | evaluate_source_references(     policy: &SourceReferencePolicy,     references: &[SourceReference],     implementation_citations: &[String], ) -> SourceReferenceReview |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/models/source_reference.rs -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/models/source_reference.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Whole-file source replay owns the source-reference policy models until a
      narrower schema generator can produce the full behavior surface.
```
