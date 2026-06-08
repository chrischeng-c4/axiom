---
id: sdd-interfaces-services-review-service-phase-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Review Service Phase Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/services/review_service.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ReviewInput` | projects/agentic-workflow/src/services/review_service.rs | struct | pub | 99 |  |
| `VALID_FILES` | projects/agentic-workflow/src/services/review_service.rs | constant | pub | 20 |  |
| `review_phase_transition` | projects/agentic-workflow/src/services/review_service.rs | function | pub | 62 | review_phase_transition(artifact: &str, verdict: &str) -> Option<StatePhase> |
| `write_review` | projects/agentic-workflow/src/services/review_service.rs | function | pub | 131 | write_review(input: ReviewInput, project_root: &Path) -> Result<String> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap review-service-phase-matrix -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/review_service.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:review-service-phase-matrix>"
    description: "Source template owns review imports, valid artifacts, and phase transitions."
```
