---
change_id: genesis-325-329
type: codebase_context
created_at: 2026-02-14T09:50:55.400418+00:00
updated_at: 2026-02-14T09:50:55.400418+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - prism_symbols
  - prism_references
---

# Codebase Context

## Analyzed Files

- **crates/cclab-aurora/src/generators/mod.rs** — Generator module entry point (FastAPI, Express, Axum)
  - symbols: `FastApiGenerator`, `ExpressGenerator`, `AxumGenerator`
- **crates/cclab-aurora/src/generators/fastapi.rs** — FastAPI code generator - transforms SchemaIR to Python FastAPI project
  - symbols: `FastApiGenerator`, `generate`, `build_context`, `render_templates`
- **crates/cclab-aurora/src/generators/express.rs** — Express.js code generator - transforms SchemaIR to TypeScript Express project
  - symbols: `ExpressGenerator`, `generate`
- **crates/cclab-aurora/src/generators/axum.rs** — Axum code generator - transforms SchemaIR to Rust Axum project
  - symbols: `AxumGenerator`, `generate`, `transform_context`
- **crates/cclab-aurora/src/engine/mod.rs** — Tera template engine wrapper with custom filters (pascal_case, camel_case, snake_case, kebab_case)
  - symbols: `TemplateEngine`, `render`, `register_filter`
- **crates/cclab-aurora/src/schema/mod.rs** — JSON Schema core types and parser (Draft 7 + 2020-12)
  - symbols: `JsonSchema`, `Schema`, `SchemaType`
- **crates/cclab-aurora/src/schema/types.rs** — JSON Schema type definitions
  - symbols: `JsonSchema`, `SchemaType`, `SchemaProperty`
- **crates/cclab-aurora/src/schema/parser.rs** — JSON Schema parser
  - symbols: `parse_schema`
- **crates/cclab-aurora/src/validator/mod.rs** — Spec completeness validator for JSON Schemas
  - symbols: `validate_schema`, `ValidationIssue`, `ValidationResult`, `Severity`
- **crates/cclab-aurora/src/specs/openapi.rs** — OpenAPI 3.1 spec generation
  - symbols: `ApiInfo`, `Server`, `HttpMethod`, `Parameter`, `RequestBody`, `ApiResponse`
- **crates/cclab-aurora/src/diagrams/flowchart_plus/schema.rs** — Flowchart+ schema with SemanticType for codegen
  - symbols: `FlowchartDef`, `NodeDef`, `SemanticType`, `EdgeDef`
- **crates/cclab-aurora/src/diagrams/class_plus/schema.rs** — Class+ schema with DDD stereotypes
  - symbols: `ClassDef`, `Stereotype`
- **crates/cclab-aurora/src/diagrams/erd_plus/schema.rs** — ERD+ schema with FK/PK validation
  - symbols: `ErdDef`, `EntityDef`, `AttributeDef`
- **crates/cclab-aurora/src/diagrams/sequence_plus/schema.rs** — Sequence+ schema with loops/alt blocks
  - symbols: `SequenceDef`, `Participant`, `Message`
- **crates/cclab-aurora/src/mcp/tools.rs** — Aurora MCP tool definitions
  - symbols: `AuroraTools`, `list`, `call_tool`, `is_aurora_tool`
- **crates/cclab-aurora/src/lib.rs** — Aurora crate root - exports diagrams, specs, mcp, schema, engine, generators, validator
- **crates/cclab-prism/src/gen/mod.rs** — Prism code generation module entry - per-crate generators
  - symbols: `CodeGenerator`, `GenContext`, `GeneratedCode`
- **crates/cclab-prism/src/gen/traits.rs** — CodeGenerator trait definition
  - symbols: `CodeGenerator`, `can_generate`, `generate`
- **crates/cclab-prism/src/spec/ir.rs** — Specification IR for code generation input
  - symbols: `SpecIR`
- **crates/cclab-prism/src/gen/python/** — Python generators (Shield, Titan, Nebula, Photon, Quasar)
  - symbols: `ShieldGenerator`, `TitanGenerator`, `NebulaGenerator`, `PhotonGenerator`, `QuasarGenerator`
- **crates/cclab-prism/src/gen/rust/** — Rust generators (Serde, Axum, SQLx, Reqwest)
  - symbols: `SerdeGenerator`, `AxumGenerator`, `SqlxGenerator`, `ReqwestGenerator`
- **crates/cclab-prism/src/mcp/tools.rs** — Prism MCP tool definitions (18 tools including prism_generate_from_spec)
  - symbols: `PrismTools`, `prism_generate_from_spec`, `prism_generate_state_machine`
- **crates/cclab-prism/src/types/codegen.rs** — Code generation helpers (24KB)
- **crates/cclab-prism/Cargo.toml** — Prism dependencies - includes cclab-aurora dependency
- **crates/cclab-genesis/src/mcp/tools/run_change/implement.rs** — Genesis implement phase handler - per-task loop and legacy path
  - symbols: `handle`, `Action`
- **crates/cclab-genesis/src/mcp/tools/run_change/mod.rs** — Genesis run_change dispatcher - routes phases to handlers
  - symbols: `handle_run_change`
- **crates/cclab-genesis/src/services/implementation_service.rs** — Genesis implementation tracking service

## Prism Results

- **prism_symbols** (query: `Aurora generators module`)
  - Found FastApiGenerator, ExpressGenerator, AxumGenerator in crates/cclab-aurora/src/generators/. Each implements generate() taking SchemaIR + options. Uses TemplateEngine from engine/ module.
- **prism_symbols** (query: `Prism gen module`)
  - Found CodeGenerator trait in gen/traits.rs with name(), can_generate(SpecIR), generate(SpecIR, GenContext). Python generators: Shield, Titan, Nebula, Photon, Quasar. Rust generators: Serde, Axum, SQLx, Reqwest.
- **prism_symbols** (query: `Aurora Plus diagram schemas`)
  - All Plus diagrams (flowchart_plus, class_plus, erd_plus, sequence_plus, etc.) have schema.rs defining structured input types. FlowchartDef has SemanticType per node. ClassDef has Stereotype (entity, valueObject, aggregate).
- **prism_references** (query: `cclab-aurora usage from cclab-prism`)
  - Prism Cargo.toml has cclab-aurora = { path = "../cclab-aurora" }. Used in spec/ module for diagram generation and in MCP handlers for prism_code_to_mermaid and prism_spec_to_mermaid tools.
- **prism_symbols** (query: `Genesis implement phase`)
  - implement.rs has handle() function with Action enum: BeginImplementation, ImplementTask, ReviewTask, ReviseTask, TaskTerminalFailure, AllTasksDone, plus legacy variants. Per-task loop reads tasks.md and routes to appropriate action.

## Dependency Graph

- cclab-prism depends on cclab-aurora (Cargo.toml)
- Aurora generators/ depends on Aurora engine/ (TemplateEngine) and schema/ (JsonSchema)
- Aurora generators/ depends on Aurora specs/ (OpenAPI types for IR)
- Prism gen/ depends on Prism spec/ir.rs (SpecIR)
- Prism mcp/ calls gen/ for prism_generate_from_spec
- Genesis implement.rs reads tasks.md and calls genesis_update_state, does NOT call Prism tools directly
