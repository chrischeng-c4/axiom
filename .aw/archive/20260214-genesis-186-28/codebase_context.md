---
change_id: genesis-186-28
type: codebase_context
created_at: 2026-02-14T03:35:02.472303+00:00
updated_at: 2026-02-14T03:35:02.472303+00:00
iteration: 5
complexity: high
stage: codebase
prism_tools_used:
  - prism_symbols
---

# Codebase Context

## Analyzed Files

- **crates/cclab-genesis/src/mcp/tools/analyze.rs** — Implementation of genesis_analyze_code_for_spec tool. Performs tree-sitter analysis of Python, TS, and Rust code.
  - symbols: `execute`, `analyze_python`, `analyze_typescript`, `analyze_rust`, `detect_spec_type`, `generate_suggestions`
- **crates/cclab-genesis/src/mcp/tools/spec.rs** — Tool layer for genesis_create_spec and genesis_review_spec. Wires arguments to spec_service. Depends on models/spec_rules.rs for SpecType/ApiSpecType parsing.
  - symbols: `definition`, `execute`, `review_spec_definition`, `execute_review_spec`
- **crates/cclab-genesis/src/mcp/tools/run_change/spec.rs** — Orchestration of spec creation/review. Handles phase transitions and revision count tracking. Directs to tasks.rs if specs are complete.
  - symbols: `handle`, `Action`
- **crates/cclab-genesis/src/mcp/tools/run_change/tasks.rs** — Orchestration of task generation. Final step of the plan stage. Depends on helpers.rs for count_spec_files.
  - symbols: `handle`, `Action`
- **crates/cclab-genesis/src/mcp/tools/run_change/helpers.rs** — Shared helpers for run_change tools, including verdict parsing and mainthread_must_fix prompt generation.
  - symbols: `phase_to_string`, `mainthread_must_fix`, `analyze_specs`, `get_last_review_verdict`
- **crates/cclab-genesis/src/services/spec_service.rs** — Service layer for creating spec files, rendering diagrams via cclab-aurora, and validating spec types.
  - symbols: `create_spec`, `validate_spec_type_requirements`, `render_diagram`
- **crates/cclab-genesis/src/models/spec_rules.rs** — Rules for spec formatting, required diagrams, and API spec types based on SpecType.
  - symbols: `SpecType`, `DiagramType`, `ApiSpecType`, `SpecFormatRules`

## Prism Results

- **prism_symbols** (query: `analyze.rs symbols`)
  - Found 83 symbols including execute and generate_suggestions
- **prism_symbols** (query: `mcp/tools/spec.rs symbols`)
  - Found 18 symbols including execute and execute_review_spec
- **prism_symbols** (query: `run_change/spec.rs symbols`)
  - Found 9 symbols including handle and Action enum
- **prism_symbols** (query: `tasks.rs symbols`)
  - Found 11 symbols including handle and Action enum
- **prism_symbols** (query: `helpers.rs symbols`)
  - Found 15 symbols including mainthread_must_fix
- **prism_symbols** (query: `spec_service.rs symbols`)
  - Found 22 symbols including create_spec and render_diagram
- **prism_symbols** (query: `spec_rules.rs symbols`)
  - Found 25 symbols including SpecType and DiagramType enums

## Dependency Graph

- mcp/tools/analyze.rs -> detect_spec_type
- mcp/tools/analyze.rs -> generate_suggestions
- mcp/tools/run_change/spec.rs -> mcp/tools/run_change/helpers.rs
- mcp/tools/run_change/spec.rs -> mcp/tools/run_change/tasks.rs
- mcp/tools/run_change/tasks.rs -> mcp/tools/run_change/helpers.rs
- mcp/tools/spec.rs -> services/spec_service.rs
- mcp/tools/spec.rs -> models/spec_rules.rs
- services/spec_service.rs -> models/spec_rules.rs
- services/spec_service.rs -> cclab-aurora
