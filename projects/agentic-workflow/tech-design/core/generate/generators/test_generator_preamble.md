---
id: sdd-generate-generators-test-generator-preamble
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# TestGenerator Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/generators/test_generator.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CoverageIssue` | projects/agentic-workflow/src/generate/generators/test_generator.rs | struct | pub | 41 |  |
| `TestGenError` | projects/agentic-workflow/src/generate/generators/test_generator.rs | enum | pub | 53 |  |
| `TestGenResult` | projects/agentic-workflow/src/generate/generators/test_generator.rs | struct | pub | 65 |  |
| `TestGenerator` | projects/agentic-workflow/src/generate/generators/test_generator.rs | struct | pub | 76 |  |
| `generate` | projects/agentic-workflow/src/generate/generators/test_generator.rs | function | pub | 96 | generate(&self, def: &RequirementDiagramDef) -> Result<TestGenResult, TestGenError> |
| `new` | projects/agentic-workflow/src/generate/generators/test_generator.rs | function | pub | 91 | new(strict: bool) -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap generate-generators-test-generator-preamble -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/generators/test_generator.rs -->
```rust

//! Test scaffold generator for RequirementPlus diagrams
//!
//! Maps `RequirementDiagramDef` → pytest test files following the
//! Requirement+ → Test mapping contract in `code-generator-contract.md`.
//!
//! # Mapping rules
//!
//! | Source | Target |
//! |--------|--------|
//! | requirement with `verifymethod: Test` | `class TestR{id}_{name}` |
//! | `Scenario -verifies-> R` | `def test_{scenario_name}(self)` |
//! | `Module -satisfies-> R` | `import` statement at file top |
//! | `R2 -derives-> R1` | comment: "R2 depends on R1; run R1 tests first" |
//! | `risk: High` | `@pytest.mark.critical` marker |
//! | `verifymethod: Inspection` | `# TODO: Manual inspection required` (no function) |
//!
//! # Safe defaults (Q2)
//!
//! Test functions contain only `# Given/When/Then` comments + `pass  # TODO: implement`.
//! No NLP heuristics are applied.

use crate::generate::diagrams::{
    ReqRelationshipTypePlus, RequirementDefPlus, RequirementDiagramDef, RiskLevelPlus,
    VerificationMethodPlus,
};
use std::collections::{BTreeMap, HashMap};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/generators/test_generator.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:generate-generators-test-generator-preamble>"
    description: "Source template owns module docs, imports, and the public types section header."
```
