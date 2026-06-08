---
change_id: genesis-372
type: codebase_context
created_at: 2026-02-14T17:06:27.404972+00:00
updated_at: 2026-02-14T17:06:27.404972+00:00
iteration: 3
complexity: high
stage: codebase
prism_tools_used:
  - prism_symbols
---

# Codebase Context

## Analyzed Files

- **crates/cclab-aurora/src/spec_ir/types.rs** — Current SpecIR definition as Rust enum with 6 variants and serde serialization
  - symbols: `SpecIR (enum: Api, FlowchartPlus, ClassPlus, ErdPlus, SequencePlus, RequirementPlus)`, `SpecMetadata (struct: source_path, spec_group, spec_id, tags)`, `SpecBundle (struct: specs, dependencies, metadata)`, `BundleMetadata (struct: change_id, description)`
- **crates/cclab-aurora/src/spec_ir/mod.rs** — SpecIR module entry point, re-exports types
  - symbols: `mod types`
- **crates/cclab-aurora/src/generators/fastapi.rs** — Aurora generator for Python/FastAPI using Tera templates
  - symbols: `FastAPIGenerator`
- **crates/cclab-aurora/src/generators/express.rs** — Aurora generator for TypeScript/Express using Tera templates
  - symbols: `ExpressGenerator`
- **crates/cclab-aurora/src/generators/axum.rs** — Aurora generator for Rust/Axum using Tera templates
  - symbols: `AxumGenerator`
- **crates/cclab-aurora/src/generators/common.rs** — Common utilities shared across Aurora generators
  - symbols: `shared generator utilities`
- **crates/cclab-aurora/src/validator/mod.rs** — Aurora validator module entry point
  - symbols: `mod completeness`
- **crates/cclab-aurora/src/validator/completeness.rs** — Schema completeness validator for JsonSchema structure
  - symbols: `Severity (enum: Error, Warning)`, `ValidationIssue (struct: path, message, severity)`, `ValidationResult`, `validate_schema(schema: &JsonSchema)`
- **crates/cclab-prism/src/gen/traits.rs** — Prism code generation trait and types, accepts SpecIR via can_generate/generate
  - symbols: `CodeGenerator (trait: name, can_generate, generate, generate_data_models, generate_rest_api, generate_event_api, generate_state_machine, generate_control_flow)`, `GenContext (struct: stack, module_name, etc.)`, `GeneratedCode (struct: name, content, imports, language)`, `TechStack (enum: 12 variants)`, `Language (enum: Python, Rust, TypeScript)`
- **crates/cclab-prism/src/gen/registry.rs** — Registry dispatches SpecIR to matching generator via can_generate()
  - symbols: `GeneratorRegistry (struct: generators)`, `register(), find(spec: &SpecIR), generate(spec, ctx), list()`
- **crates/cclab-prism/src/gen/mod.rs** — Prism codegen module entry point
  - symbols: `mod traits, registry, python, rust`
- **crates/cclab-genesis/src/mcp/tools/spec.rs** — genesis_create_spec and genesis_review_spec MCP tool implementations
  - symbols: `definition() -> ToolDefinition`, `execute(args, project_root)`, `review_spec_definition() -> ToolDefinition`, `execute_review_spec(args, project_root)`, `parse_changes(args)`, `parse_string_array_opt(args, field)`
- **crates/cclab-genesis/src/mcp/tools/run_change/implement.rs** — Genesis implement phase routing, including codegen-eligible task detection
  - symbols: `handle()`, `TaskContext`, `Action (enum: BeginImplementation, ImplementTask, ImplementTaskWithCodegen, ReviewTask, ReviseTask, TaskTerminalFailure, AllTasksDone, Legacy*)`, `determine_action()`, `build_response()`
- **crates/cclab-genesis/src/mcp/tools/run_change/task_graph.rs** — Task graph helpers, topological sort, codegen eligibility check
  - symbols: `TaskInfo (struct: id, status, depends_on, spec_ref)`, `parse_task_blocks()`, `build_task_execution_order()`, `find_next_task()`, `find_completed_tasks()`, `is_codegen_eligible()`
- **crates/cclab-genesis/src/mcp/tools/run_change/spec.rs** — Genesis run_change spec flow routing, emits prompts referencing genesis_create_spec tool
  - symbols: `handle()`, `Action (enum: CreateSpec, ReviewSpec, ReviseSpec)`
- **crates/cclab-genesis/src/mcp/tools/run_change/mod.rs** — Main run_change entry point, state machine routing
  - symbols: `definition()`, `execute()`, `route()`, `action_to_artifact()`, `add_executor_info()`
- **crates/cclab-genesis/src/mcp/tools/run_change/helpers.rs** — Shared helpers for all run_change flow modules
  - symbols: `re-exports from task_graph.rs`, `read_tasks_md()`, `extract_verdict()`, `GENESIS_RUN_CHANGE_TOOL, GENESIS_AGENT_TOOL`

## Prism Results

- **prism_symbols** (query: `crates/cclab-aurora/src/spec_ir/types.rs`)
  - SpecIR enum with 6 variants. SpecMetadata and SpecBundle structs. From impls for each Aurora diagram type.
- **prism_symbols** (query: `crates/cclab-prism/src/gen/traits.rs`)
  - CodeGenerator trait with name(), can_generate(&SpecIR), generate(&SpecIR, &GenContext). TechStack (12 variants), Language (3 variants), GenContext builder.
- **prism_symbols** (query: `crates/cclab-prism/src/gen/registry.rs`)
  - GeneratorRegistry holds Vec<Box<dyn CodeGenerator>>. find() returns first generator where can_generate() is true.
- **prism_symbols** (query: `crates/cclab-genesis/src/mcp/tools/spec.rs`)
  - genesis_create_spec: definition() + execute(). genesis_review_spec: review_spec_definition() + execute_review_spec(). Creates spec markdown files with frontmatter and tag-union validation.
- **prism_symbols** (query: `crates/cclab-genesis/src/mcp/tools/run_change/implement.rs`)
  - Action enum includes ImplementTaskWithCodegen. determine_action() checks is_codegen_eligible(). build_response() constructs Prism MCP tool references.
- **prism_symbols** (query: `crates/cclab-genesis/src/mcp/tools/run_change/task_graph.rs`)
  - TaskInfo with spec_ref. is_codegen_eligible() checks spec design_elements.
- **prism_symbols** (query: `crates/cclab-genesis/src/mcp/tools/run_change/spec.rs`)
  - Action enum: CreateSpec, ReviewSpec, ReviseSpec. handle() routes to spec flow actions.
- **prism_symbols** (query: `crates/cclab-aurora/src/validator/completeness.rs`)
  - Severity enum, ValidationIssue/ValidationResult structs. validate_schema() validates JsonSchema structure recursively.

## Dependency Graph

- cclab-aurora/spec_ir/types.rs defines SpecIR enum imported by cclab-prism/gen/registry.rs
- cclab-prism/gen/registry.rs imports SpecIR from cclab-aurora
- cclab-prism/gen/traits.rs CodeGenerator trait uses SpecIR for can_generate/generate dispatch
- cclab-genesis/run_change/implement.rs calls task_graph::is_codegen_eligible() for routing
- cclab-genesis/run_change/task_graph.rs is_codegen_eligible checks spec files on disk
- cclab-genesis/run_change/spec.rs emits prompts referencing genesis_create_spec tool implemented in mcp/tools/spec.rs
- cclab-genesis/mcp/tools/spec.rs execute() creates spec markdown files with tag-union validation
- cclab-genesis/run_change/helpers.rs re-exports task_graph types (TaskInfo, parse_task_blocks, etc.)
- cclab-aurora/validator/completeness.rs validate_schema() takes &JsonSchema from cclab-aurora/src/schema/types.rs
- Aurora generators (fastapi, express, axum) are standalone, use Tera templates, not integrated with SpecIR dispatch
