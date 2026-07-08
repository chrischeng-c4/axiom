---
id: sdd-generate-generators-test-generator-runtime
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# TestGenerator Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/generate/generators/test_generator.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CoverageIssue` | apps/agentic-workflow/src/generate/generators/test_generator.rs | struct | pub | 41 |  |
| `TestGenError` | apps/agentic-workflow/src/generate/generators/test_generator.rs | enum | pub | 53 |  |
| `TestGenResult` | apps/agentic-workflow/src/generate/generators/test_generator.rs | struct | pub | 65 |  |
| `TestGenerator` | apps/agentic-workflow/src/generate/generators/test_generator.rs | struct | pub | 76 |  |
| `generate` | apps/agentic-workflow/src/generate/generators/test_generator.rs | function | pub | 96 | generate(&self, def: &RequirementDiagramDef) -> Result<TestGenResult, TestGenError> |
| `new` | apps/agentic-workflow/src/generate/generators/test_generator.rs | function | pub | 91 | new(strict: bool) -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap generate-generators-test-generator-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/generate/generators/test_generator.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:generate-generators-test-generator-runtime>"
    description: "Source template owns the test generator runtime and regression tests."
```
