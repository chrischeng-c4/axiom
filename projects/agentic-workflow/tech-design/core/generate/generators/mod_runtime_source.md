---
id: sdd-generate-generators-mod-runtime-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Generator Module Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generators/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `GeneratorArgs` | projects/agentic-workflow/src/generators/mod.rs | struct | pub | 55 |  |
| `async_api` | projects/agentic-workflow/src/generators/mod.rs | module | pub | 39 |  |
| `changes` | projects/agentic-workflow/src/generators/mod.rs | module | pub | 21 |  |
| `db_model` | projects/agentic-workflow/src/generators/mod.rs | module | pub | 30 |  |
| `dependency` | projects/agentic-workflow/src/generators/mod.rs | module | pub | 31 |  |
| `doc` | projects/agentic-workflow/src/generators/mod.rs | module | pub | 25 |  |
| `flowchart` | projects/agentic-workflow/src/generators/mod.rs | module | pub | 32 |  |
| `frontend` | projects/agentic-workflow/src/generators/mod.rs | module | pub | 44 |  |
| `generate_section` | projects/agentic-workflow/src/generators/mod.rs | function | pub | 174 | generate_section(args: &GeneratorArgs) -> Option<String> |
| `get_generator` | projects/agentic-workflow/src/generators/mod.rs | function | pub | 134 | get_generator(section_type: SectionType) -> Option<Box<dyn Generator>> |
| `mindmap` | projects/agentic-workflow/src/generators/mod.rs | module | pub | 33 |  |
| `new` | projects/agentic-workflow/src/generators/mod.rs | function | pub | 71 | new(section_type: SectionType) -> Self |
| `overview` | projects/agentic-workflow/src/generators/mod.rs | module | pub | 22 |  |
| `requirements` | projects/agentic-workflow/src/generators/mod.rs | module | pub | 26 |  |
| `rest_api` | projects/agentic-workflow/src/generators/mod.rs | module | pub | 40 |  |
| `rpc_api` | projects/agentic-workflow/src/generators/mod.rs | module | pub | 41 |  |
| `scenarios` | projects/agentic-workflow/src/generators/mod.rs | module | pub | 27 |  |
| `sequence` | projects/agentic-workflow/src/generators/mod.rs | module | pub | 34 |  |
| `state_machine` | projects/agentic-workflow/src/generators/mod.rs | module | pub | 35 |  |
| `test_plan` | projects/agentic-workflow/src/generators/mod.rs | module | pub | 36 |  |
| `with_sdd_id` | projects/agentic-workflow/src/generators/mod.rs | function | pub | 80 | with_sdd_id(mut self, sdd_id: impl Into<String>) -> Self |
| `with_sdd_refs` | projects/agentic-workflow/src/generators/mod.rs | function | pub | 86 | with_sdd_refs(mut self, refs: Vec<String>) -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap generate-generators-mod-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generators/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:generate-generators-mod-runtime>"
    description: "Source template owns generator builders, trait, dispatch, and tests."
```
