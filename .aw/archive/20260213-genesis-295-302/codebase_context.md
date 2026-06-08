---
change_id: genesis-295-302
type: codebase_context
created_at: 2026-02-13T07:42:37.686337+00:00
updated_at: 2026-02-13T07:42:37.686337+00:00
iteration: 1
complexity: medium
stage: codebase
prism_tools_used:
  - prism_symbols
  - grep_search
---

# Codebase Context

## Analyzed Files

- **crates/cclab-genesis/src/mcp/tools/run_change/mod.rs** — Orchestrator for the unified workflow tool, routing by StatePhase and adding executor info.
  - symbols: `route`, `execute`, `add_executor_info`, `action_to_artifact`
- **crates/cclab-genesis/src/services/spec_service.rs** — Business logic for spec creation, including tag resolution and requirement validation.
  - symbols: `create_spec`, `resolve_tags`, `validate_spec_type_requirements`, `validate_api_spec`
- **crates/cclab-genesis/src/services/proposal_service.rs** — Handles both v1 (legacy) and v2 proposal generation.
  - symbols: `create_proposal`, `create_v1_proposal`, `create_v2_proposal`, `validate_input`
- **crates/cclab-genesis/src/models/spec_rules.rs** — Source of truth for spec type requirements (diagrams, API specs) and markdown formatting.
  - symbols: `SpecType`, `DiagramType`, `ApiSpecType`, `SpecFormatRules`
- **crates/cclab-genesis/src/mcp/tools/fetch_issues.rs** — Fetches GitHub issues and parses descriptions for dependency links.
  - symbols: `extract_dependencies`, `topological_sort`, `update_state_dag`

## Prism Results

- **prism_symbols** (query: `Tag validation logic`)
  - Tags are resolved by combining auto-tags from SpecType (e.g., http-api -> [api, http]) with explicit tags provided in input.
- **prism_symbols** (query: `v1 legacy paths`)
  - ProposalService branches on version >= 2; version 1 uses a summary-based markdown layout with legacy impact fields.
- **prism_symbols** (query: `run_change routing`)
  - Routing is strictly phase-driven, mapping StatePhase to specific flow handlers (clarify, explore, proposal, spec, etc.).

## Dependency Graph

- crates/cclab-genesis/src/mcp/tools/run_change/mod.rs -> crates/cclab-genesis/src/mcp/tools/run_change/{proposal.rs, spec.rs, tasks.rs, implement.rs, merge.rs}
- crates/cclab-genesis/src/mcp/tools/run_change/proposal.rs -> crates/cclab-genesis/src/services/proposal_service.rs
- crates/cclab-genesis/src/mcp/tools/run_change/spec.rs -> crates/cclab-genesis/src/services/spec_service.rs
- crates/cclab-genesis/src/services/spec_service.rs -> crates/cclab-genesis/src/models/spec_rules.rs
