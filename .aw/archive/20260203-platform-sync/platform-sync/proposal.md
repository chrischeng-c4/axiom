---
id: platform-sync
type: proposal
version: 1
created_at: 2026-02-03T03:51:40.686899+00:00
updated_at: 2026-02-03T03:51:40.686899+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add a publish-only Platform Sync MCP tool that syncs Genesis change artifacts to GitHub using gh CLI auth"
history:
  - timestamp: 2026-02-03T03:51:40.686899+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-02-03T03:51:46.735691+00:00
    agent: "codex:deep"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-02-03T03:51:55.511499+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 7
  new_files: 2---

<proposal>

# Change: platform-sync

## Summary

Add a publish-only Platform Sync MCP tool that syncs Genesis change artifacts to GitHub using gh CLI auth

## Why

Publishing Genesis change artifacts to external platforms is currently manual and inconsistent. A dedicated Platform Sync MCP tool enables reliable one-way publishing to GitHub (the prioritized platform) while setting up a provider interface that can later support GitLab/Jira, without introducing new credential storage.

## What Changes

- Introduce project-level platform config in `cclab/config.yaml` to specify platform type, repo, and token source.
- Support reading tokens from `.env` files via `envfile` and `envfield` config options.
- Introduce a platform sync service that builds a deterministic markdown payload from change artifacts (proposal/specs/tasks) and computes a payload hash for idempotent updates.
- Add a GitHub provider that uses GitHub API when token is available, with fallback to `gh` CLI when not.
- Persist sync metadata in the change directory (SYNC.yaml) to track updates and enable future multi-platform sync.
- Expose a new MCP tool (`genesis_platform_sync`) that reads config and syncs automatically.
- Add unit tests for config loading, env file parsing, payload rendering, and metadata persistence.

## Impact

- **Scope**: minor
- **Affected Files**: ~7
- **New Files**: ~2
- Affected code: `crates/cclab-genesis/src/services/mod.rs`, `crates/cclab-genesis/src/services/platform_sync_service.rs`, `crates/cclab-genesis/src/services/platform_sync/github_provider.rs`, `crates/cclab-genesis/src/mcp/tools/mod.rs`, `crates/cclab-genesis/src/mcp/tools/platform_sync.rs`, `cclab/specs/cclab-genesis/shared/tools.openrpc.json`, `crates/cclab-genesis/tests/platform_sync_test.rs`
- **Breaking Changes**: None. Adds a new MCP tool and internal service; existing workflows remain unchanged.

</proposal>
