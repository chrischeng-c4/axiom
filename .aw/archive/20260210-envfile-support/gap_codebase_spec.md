---
change_id: envfile-support
type: gap_codebase_spec
created_at: 2026-02-10T02:29:15.832267+00:00
updated_at: 2026-02-10T02:29:15.832267+00:00
---

# Gap Analysis: Codebase vs Spec

## Identified Gaps

1. **Missing Config Fields**:
    - **Spec**: `cclab-genesis/orchestrator` mentions that workflow stages must have configurable agent lists in `config.toml`. It doesn't mention `envfile`.
    - **Codebase**: `crates/cclab-genesis/src/models/change.rs` contains `WorkflowConfig`, `GeminiConfig`, `CodexConfig`, and `ClaudeConfig` structs, none of which currently have an `envfile` field.
    - **Gap**: We need to add `envfile: Option<String>` to these structs to support the new requirement.

2. **Environment Variable Loading**:
    - **Spec**: `cclab-shield/shield-settings-management` defines a pattern for loading `.env` files using `dotenvy`.
    - **Codebase**: `crates/cclab-genesis/src/mcp/tools/agent.rs` (specifically `execute_streaming`) currently initializes an empty `HashMap` for environment variables and passes it to `ScriptRunner`.
    - **Gap**: Logic to load `.env` files (global and per-provider) and merge them into the `HashMap` is missing in `agent.rs`.

3. **Variable Substitution**:
    - **Clarifications**: Q1 specifies that the envfile format should support standard dotenv with variable substitution (e.g., `${VAR}`).
    - **Gap**: The implementation must ensure that whatever library or logic is used for loading `.env` files supports this substitution, or implement it manually if `dotenvy` doesn't provide it out of the box (though `dotenvy` generally follows standard dotenv behavior).

4. **Configuration Structure Discrepancy**:
    - **Spec**: `cclab-genesis/orchestrator` shows a simplified `[workflow.stages.plan]` structure in its example.
    - **Codebase**: `crates/cclab-genesis/src/models/change.rs` uses a more granular `[workflow.agents]` structure (represented by `AgentsConfig` struct) with many specific artifact keys (e.g., `explore_spec`, `create_proposal`).
    - **Gap**: While not strictly a gap for `envfile-support`, it's important to realize that the global `envfile` should probably go into the `[workflow]` section as requested in Clarification Q2, which maps to `WorkflowConfig` struct in the codebase.
