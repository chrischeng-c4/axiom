// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/gen/rust/mod.md#source
// CODEGEN-BEGIN
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
    generate_test_plan, generate_test_plan_from_markdown, generate_unit_tests_from_mermaid,
    MarkdownTest, MarkdownTestPlanOutput, ScenarioRef, TestElement, TestPlanGenOutput,
};
pub use tests_gen::{generate_e2e_tests, generate_tests, TestsGenOutput};

// CODEGEN-END
