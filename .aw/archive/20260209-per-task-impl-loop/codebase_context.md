---
change_id: per-task-impl-loop
type: codebase_context
created_at: 2026-02-09T08:41:01.968382+00:00
updated_at: 2026-02-09T08:41:01.968382+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - read_file
  - search_file_content
  - list_directory
---

# Codebase Context

## Analyzed Files

- **crates/cclab-genesis/src/mcp/tools/run_change/implement.rs** — Primary refactoring target — single-pass state machine routing Planned→Implementing→Implemented→ImplReviewed→ImplRevised→ImplApproved
  - symbols: `handle()`, `Action enum (Implement/Resume/ReviewImpl/HandleReview)`, `build_implement_prompt()`, `build_review_prompt()`
- **crates/cclab-genesis/src/mcp/tools/run_change/mod.rs** — Top-level route() dispatches phases to handler modules; add_executor_info() attaches mainthread_instruction; action_to_artifact() maps actions to WorkflowArtifact
  - symbols: `route()`, `add_executor_info()`, `action_to_artifact()`
- **crates/cclab-genesis/src/models/task_graph.rs** — TaskGraph parses tasks.md into layers (data/logic/integration/testing) with topological sort; provides get_execution_order() for dependency-aware sequencing
  - symbols: `TaskGraph`, `TaskNode`, `TaskStatus`, `get_execution_order()`, `parse()`
- **crates/cclab-genesis/src/state/manager.rs** — StateManager reads/writes STATE.yaml; tracks phase, revision_counts, iteration; currently lacks current_task_id field
  - symbols: `StateManager`, `load()`, `save()`, `set_phase()`, `increment_revision_count()`
- **crates/cclab-genesis/src/models/frontmatter.rs** — StatePhase enum (37 variants) + State struct; defines is_decide_phase/is_plan_phase etc; State needs current_task_id field
  - symbols: `StatePhase`, `State`, `is_decide_phase()`, `is_plan_phase()`, `is_implement_phase()`
- **crates/cclab-genesis/src/mcp/tools/state_update.rs** — Phase transition validation (parse_phase, phase_order, validate_transition); revision counter auto-increment; currently no ImplApproved→Implementing loop support
  - symbols: `parse_phase()`, `phase_order()`, `validate_transition()`, `execute()`
- **crates/cclab-genesis/src/mcp/tools/run_change/helpers.rs** — Shared helpers: extract_review_info() parses REVIEW_IMPL.md verdict+issues; update_phase_from_review() decides approve/revise/fallback; read_tasks_md()
  - symbols: `extract_review_info()`, `update_phase_from_review()`, `read_tasks_md()`, `ReviewInfo`
- **crates/cclab-genesis/src/mcp/tools/implementation.rs** — genesis_review_implementation MCP tool — creates REVIEW_IMPL.md with verdict (PASS/FAIL/PARTIAL)
  - symbols: `definition()`, `execute()`
- **crates/cclab-genesis/src/services/implementation_service.rs** — Service layer for implementation review — writes REVIEW_IMPL.md, parses verdict from markdown
  - symbols: `create_review()`, `ImplementationReview`
- **crates/cclab-genesis/src/models/verification.rs** — Verification model with TaskStatus enum mapping to markdown symbols (✅/🔲/⏳); used for tasks.md checkbox tracking
  - symbols: `TaskStatus`, `VerificationResult`, `to_markdown_symbol()`
- **crates/cclab-genesis/src/mcp/tools/run_change/tasks.rs** — Tasks generation handler — generates tasks.md from specs; currently one-shot generation in plan stage
  - symbols: `handle()`

## Prism Results

- **search_file_content** (query: `update_phase_from_review`)
  - Found in 5 locations: helpers.rs (definition), implement.rs (usage for impl review), explore_spec.rs, explore_knowledge.rs, explore_codebase.rs (usage for context review). All follow same pattern: extract verdict → approve/revise/fallback.
- **search_file_content** (query: `genesis_review_implementation`)
  - Found in 12 locations: tool definition in implementation.rs, service in implementation_service.rs, prompts in implement.rs, registration in mod.rs. Single review artifact REVIEW_IMPL.md covers entire implementation.
- **search_file_content** (query: `TaskStatus`)
  - Found in 16 locations: defined in task_graph.rs (Pending/InProgress/Done/Blocked), also in verification.rs with markdown symbol mapping. Currently only used for initial task generation, not updated during implementation.

## Dependency Graph

- mod.rs::route() → implement.rs::handle() [phase-based dispatch]
- implement.rs::handle() → helpers.rs::extract_review_info() [review parsing]
- implement.rs::handle() → helpers.rs::update_phase_from_review() [state advancement]
- implement.rs → state_update.rs [via genesis_update_state MCP call in prompts]
- implement.rs → implementation.rs [via genesis_review_implementation MCP call in prompts]
- task_graph.rs::TaskGraph ← tasks.rs::handle() [task generation]
- task_graph.rs::TaskGraph — NOT currently used by implement.rs [key gap]
- state/manager.rs → frontmatter.rs::State [serialize/deserialize STATE.yaml]
- verification.rs::TaskStatus ← task_graph.rs::TaskStatus [same concept, separate models]
