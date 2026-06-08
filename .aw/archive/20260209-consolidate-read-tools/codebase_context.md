---
change_id: consolidate-read-tools
type: codebase_context
created_at: 2026-02-09T07:09:43.780413+00:00
updated_at: 2026-02-09T07:09:43.780413+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - manual_analysis
---

# Codebase Context

## Analyzed Files

- **crates/cclab-genesis/src/mcp/tools/read.rs** — PRIMARY — genesis_read_file + genesis_list_specs definitions and execute functions. Will be extended with new scope prefixes.
  - symbols: `definition()`, `execute()`, `list_specs_definition()`, `execute_list_specs()`
- **crates/cclab-genesis/src/services/file_service.rs** — PRIMARY — read_file() business logic with file match dispatch. Will be extended with knowledge:/spec:/list: scope routing.
  - symbols: `read_file()`, `list_specs()`
- **crates/cclab-genesis/src/mcp/tools/mod.rs** — PRIMARY — ToolRegistry with all_tools_vec(), stage filters, call_tool() dispatch. Remove 6 tool registrations and dispatch entries.
  - symbols: `ToolRegistry`, `all_tools_vec()`, `call_tool()`, `plan_tools()`, `challenge_tools()`, `implement_tools()`, `review_tools()`, `archive_tools()`
- **crates/cclab-genesis/src/mcp/tools/knowledge.rs** — TO REMOVE — genesis_read_knowledge + genesis_list_knowledge. Logic moves to file_service.rs.
  - symbols: `read_definition()`, `execute_read()`, `list_definition()`, `execute_list()`
- **crates/cclab-genesis/src/mcp/tools/main_spec.rs** — TO REMOVE — genesis_read_main_spec + genesis_list_main_specs. Logic moves to file_service.rs.
  - symbols: `list_definition()`, `execute_list()`, `read_definition()`, `execute_read()`
- **crates/cclab-genesis/src/mcp/tools/implementation.rs** — PARTIAL REMOVE — genesis_read_all_requirements moves to file_service.rs. Keep review/merge tools.
  - symbols: `read_all_requirements_definition()`, `execute_read_all_requirements()`
- **crates/cclab-genesis/src/services/knowledge_service.rs** — REUSE — existing read_knowledge() and list_knowledge() functions called by new file_service dispatch.
  - symbols: `read_knowledge()`, `list_knowledge()`
- **crates/cclab-genesis/src/mcp/tools/run_change/explore_spec.rs** — UPDATE PROMPTS — references genesis_list_main_specs and genesis_read_main_spec in prompt strings.
  - symbols: `handle()`
- **crates/cclab-genesis/src/mcp/tools/run_change/explore_knowledge.rs** — UPDATE PROMPTS — references genesis_list_knowledge and genesis_read_knowledge in prompt strings.
  - symbols: `handle()`
- **crates/cclab-genesis/src/mcp/tools/run_change/explore_codebase.rs** — NO CHANGE — only references genesis_read_file which stays.
  - symbols: `handle()`
- **crates/cclab-genesis/src/prompts/explore.md** — UPDATE — references genesis_read_file for clarifications (stays).
- **crates/cclab-genesis/src/prompts/create_spec.md** — UPDATE — references genesis_read_file (stays).

## Prism Results

- **manual_analysis** (query: `grep for genesis_read_knowledge, genesis_list_knowledge, genesis_read_main_spec, genesis_list_main_specs, genesis_read_all_requirements, genesis_list_specs across run_change modules and prompts`)
  - 30+ references to genesis_read_file (stays). explore_spec.rs references genesis_list_main_specs + genesis_read_main_spec. explore_knowledge.rs references genesis_list_knowledge + genesis_read_knowledge. implementation.rs has read_all_requirements. All prompt templates reference genesis_read_file. Stage filters in mod.rs register removed tools in challenge_tools, implement_tools, archive_tools.

## Dependency Graph

- read.rs -> file_service.rs (read_file, list_specs)
- knowledge.rs -> knowledge_service.rs (read_knowledge, list_knowledge)
- main_spec.rs -> filesystem (cclab/specs/)
- implementation.rs -> filesystem (cclab/changes/)
- mod.rs -> read.rs, knowledge.rs, main_spec.rs, implementation.rs (registrations)
- run_change/*.rs -> mod.rs (tool names in prompts)
