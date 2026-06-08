---
change_id: genesis-fetch-issues
type: codebase_context
created_at: 2026-02-12T02:04:07.722684+00:00
updated_at: 2026-02-12T02:04:07.722684+00:00
iteration: 2
complexity: high
stage: codebase
prism_tools_used:
  - prism_symbols
  - prism_diagnostics
  - prism_references
---

# Codebase Context

## Analyzed Files

- **crates/cclab-genesis/src/services/platform_sync/github.rs** — GitHub provider implementation with API/CLI fallback logic.
  - symbols: `GitHubProvider`, `sync`, `upsert_issue_api`, `upsert_issue_cli`, `run_gh`, `parse_issue_url`
- **crates/cclab-genesis/src/services/platform_sync/mod.rs** — Service orchestration and public API for platform synchronization.
  - symbols: `PlatformSyncService`, `new`, `load_config`, `sync`
- **crates/cclab-genesis/src/services/platform_sync/payload.rs** — Core logic for parsing markdown and building sync payloads.
  - symbols: `build_payload`, `write_issue_to_frontmatter`, `update_body_with_spec_links`, `extract_frontmatter`, `extract_github_issue`, `build_spec_payloads`
- **crates/cclab-genesis/src/services/platform_sync/config.rs** — Configuration management, token resolution, and title/label formatting.
  - symbols: `PlatformConfig`, `load`, `get_token`, `extract_scope_labels`, `format_proposal_title`
- **crates/cclab-genesis/src/services/platform_sync/types.rs** — Data transfer objects and status enums for synchronization results.
  - symbols: `SyncResult`, `SyncPayload`, `SpecPayload`, `SyncStatus`
- **crates/cclab-genesis/src/mcp/tools/platform_sync.rs** — MCP tool integration for genesis_platform_sync.
  - symbols: `definition`, `execute`

## Prism Results

- **prism_symbols** (query: `prism_symbols on platform_sync files`)
  - Extracted detailed symbol lists for github.rs, payload.rs, mod.rs, config.rs, and types.rs to confirm architectural roles.
- **prism_diagnostics** (query: `crates/cclab-genesis/src/services/platform_sync/`)
  - Found multiple .unwrap() calls in payload.rs and config.rs tests/helper functions (RS101). Identified potential borrowing optimizations (RS001).
- **prism_references** (query: `payload:: in github.rs`)
  - Confirmed direct dependency from GitHubProvider on payload::update_body_with_spec_links for cross-linking specifications in issue descriptions.

## Dependency Graph

- crates/cclab-genesis/src/services/platform_sync/mod.rs -> crates/cclab-genesis/src/services/platform_sync/github.rs
- crates/cclab-genesis/src/services/platform_sync/mod.rs -> crates/cclab-genesis/src/services/platform_sync/payload.rs
- crates/cclab-genesis/src/services/platform_sync/mod.rs -> crates/cclab-genesis/src/services/platform_sync/config.rs
- crates/cclab-genesis/src/services/platform_sync/mod.rs -> crates/cclab-genesis/src/services/platform_sync/types.rs
- crates/cclab-genesis/src/services/platform_sync/github.rs -> crates/cclab-genesis/src/services/platform_sync/payload.rs (for body updates)
- crates/cclab-genesis/src/services/platform_sync/github.rs -> crates/cclab-genesis/src/services/platform_sync/config.rs
- crates/cclab-genesis/src/services/platform_sync/github.rs -> crates/cclab-genesis/src/services/platform_sync/types.rs
- crates/cclab-genesis/src/services/platform_sync/payload.rs -> crates/cclab-genesis/src/services/platform_sync/config.rs
- crates/cclab-genesis/src/services/platform_sync/payload.rs -> crates/cclab-genesis/src/services/platform_sync/types.rs
- crates/cclab-genesis/src/mcp/tools/platform_sync.rs -> crates/cclab-genesis/src/services/platform_sync/mod.rs
