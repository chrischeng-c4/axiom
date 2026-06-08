---
change_id: genesis-fetch-issues
type: gap_codebase_spec
created_at: 2026-02-12T02:34:49.970644+00:00
updated_at: 2026-02-12T02:34:49.970644+00:00
---

# Gap Analysis: Codebase vs Spec (genesis-fetch-issues)

## 1. Spec Gaps (Specs without implementation)

### Tool: genesis_fetch_issues (HIGH)
The core tool defined in `cclab-genesis/fetch-issues` spec is entirely missing. There is no implementation in `src/mcp/tools/` nor registration in `src/mcp/tools/mod.rs`.

### Service Layer: Platform Fetch Logic (HIGH)
`GitHubProvider` in `crates/cclab-genesis/src/services/platform_sync/github.rs` only implements "push" (upsert) operations. It lacks:
- Logic to fetch issue details (body, labels, comments) via CLI or API.
- GraphQL query support for extracting dependencies (`blockedBy`, `blocking`, `subIssues`).
- Logic for related issue auto-fetching.

### Data Model: STATE.yaml DAG Section (HIGH)
The `Change` model in `crates/cclab-genesis/src/models/change.rs` does not include the `dag` field or the associated `DagState` structure required to track topological order and loop indices (`clarify_index`, `context_index`).

### Workflow: run_change Description Parsing (HIGH)
The `genesis_run_change` tool (implemented across `src/mcp/tools/run_change/*.rs`) lacks the logic to parse the `description` field for issue references (URLs, #NNN) to return the `fetch_issues` action as the first step for new changes.

### Workflow: Topological Looping (HIGH)
The implementation of `run_change` for the `clarify` and `context` phases does not yet support per-issue iteration based on DAG indices. It currently assumes a single-pass approach or simple status-based transitions.

### Artifacts: Issue & Graph Generation (MEDIUM)
Missing logic to generate the specified markdown artifacts:
- `issue_{NNN}.md` for each main and related issue.
- `dependency_graph.md` with Mermaid visualization.

## 2. Code Gaps (Code without matching spec)

### Platform Sync: Upsert Logic (LOW)
The `GitHubProvider` contains existing logic for `upsert_issue_api` and `upsert_issue_cli`. While these are functional, they are not covered by the `fetch-issues` spec (which focuses exclusively on fetching). These are likely covered by legacy or different spec files (`platform-integrations.md`) and do not conflict with the current change.
