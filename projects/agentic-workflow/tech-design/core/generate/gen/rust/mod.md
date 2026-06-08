---
id: projects-sdd-src-generate-gen-rust-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Rust Generator Module Index

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/gen/rust/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `cli` | projects/agentic-workflow/src/generate/gen/rust/mod.rs | module | pub | 6 |  |
| `config` | projects/agentic-workflow/src/generate/gen/rust/mod.rs | module | pub | 7 |  |
| `db_model` | projects/agentic-workflow/src/generate/gen/rust/mod.rs | module | pub | 8 |  |
| `interaction` | projects/agentic-workflow/src/generate/gen/rust/mod.rs | module | pub | 18 |  |
| `logic` | projects/agentic-workflow/src/generate/gen/rust/mod.rs | module | pub | 19 |  |
| `logic_emitter` | projects/agentic-workflow/src/generate/gen/rust/mod.rs | module | pub | 26 |  |
| `mamba_binding` | projects/agentic-workflow/src/generate/gen/rust/mod.rs | module | pub | 9 |  |
| `manifest` | projects/agentic-workflow/src/generate/gen/rust/mod.rs | module | pub | 10 |  |
| `readme` | projects/agentic-workflow/src/generate/gen/rust/mod.rs | module | pub | 11 |  |
| `requirement` | projects/agentic-workflow/src/generate/gen/rust/mod.rs | module | pub | 29 |  |
| `rpc_api` | projects/agentic-workflow/src/generate/gen/rust/mod.rs | module | pub | 12 |  |
| `scenario` | projects/agentic-workflow/src/generate/gen/rust/mod.rs | module | pub | 30 |  |
| `schema` | projects/agentic-workflow/src/generate/gen/rust/mod.rs | module | pub | 13 |  |
| `state_machine` | projects/agentic-workflow/src/generate/gen/rust/mod.rs | module | pub | 20 |  |
| `test_plan` | projects/agentic-workflow/src/generate/gen/rust/mod.rs | module | pub | 31 |  |
| `tests_gen` | projects/agentic-workflow/src/generate/gen/rust/mod.rs | module | pub | 15 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/gen/rust/mod.rs -->
```rust
//! Rust code generators — structural (100%) and behavioral (skeleton + markers).

// Structural generators (Category A — deterministic, 100% coverage)
pub mod cli;
pub mod config;
pub mod db_model;
pub mod mamba_binding;
pub mod manifest;
pub mod readme;
pub mod rpc_api;
pub mod schema;
#[path = "tests.rs"]
pub mod tests_gen;

// Behavioral generators (Category B — skeleton + SPEC-REF markers, 20-40% coverage)
pub mod interaction;
pub mod logic;
pub mod state_machine;

// SPIKE: minimum-viable LogicEmitter — flowchart → byte-equivalent fn body.
// Pattern 1 (linear flow with nested loops) only. See
// projects/agentic-workflow/tech-design/core/generate/gen/rust/logic-emitter.md for
// scope, limitations, and the Path B follow-up roadmap.
pub mod logic_emitter;

// Documentation generators (Category C — annotations and stubs)
pub mod requirement;
pub mod scenario;
pub mod test_plan;

pub use cli::{generate_cli, CliGenOutput};
pub use config::{generate_config, ConfigGenOutput};
pub use db_model::{generate_db_model, DbModelGenOutput};
pub use interaction::{generate_interaction, InteractionGenOutput};
pub use logic::{generate_logic, LogicGenOutput};
pub use mamba_binding::{generate_mamba_binding, MambaBindingGenOutput};
pub use manifest::{generate_manifest, ManifestGenOutput};
pub use readme::{generate_readme_symbols, ReadmeGenOutput, SymbolEntry};
pub use requirement::{
    generate_requirement_annotations, parse_requirement_annotations, RequirementAnnotation,
    RequirementAnnotationOutput,
};
pub use rpc_api::{generate_rpc_api, RpcApiGenOutput};
pub use scenario::{generate_scenarios, parse_scenarios, ScenarioDef, ScenarioGenOutput};
pub use schema::{generate_schema, SchemaGenOutput};
pub use state_machine::{generate_state_machine, snake_to_pascal, StateMachineGenOutput};
pub use test_plan::{
    generate_test_plan, generate_test_plan_from_markdown, MarkdownTest, MarkdownTestPlanOutput,
    ScenarioRef, TestElement, TestPlanGenOutput,
};
pub use tests_gen::{generate_tests, TestsGenOutput};
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/gen/rust/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete Rust generator module index.
```
