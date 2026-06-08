---
id: genesis-fetch-issues
type: exploration
created_at: 2026-02-11T17:22:00.967917+00:00
needs_clarification: false
---

# Codebase Exploration

# Exploration: genesis-fetch-issues

## Overview
The goal is to implement the `genesis_fetch_issues` MCP tool as defined in the `cclab-genesis/fetch-issues` specification. This tool will allow fetching main issues and their related issues (links, child items) from external platforms (GitHub/GitLab), extracting dependency relationships, and building an execution DAG.

## Architecture & Existing Patterns
- **Platform Sync Service**: Located in `crates/cclab-genesis/src/services/platform_sync/`. Currently handles pushing artifacts to issues.
- **GitHub Provider**: Implemented in `github.rs`, using both `reqwest` for API calls and `gh` CLI as a fallback.
- **State Management**: `STATE.yaml` tracks change progress. The spec requires adding a `dag` section to this file.
- **MCP Tools**: Tools are defined in `crates/cclab-genesis/src/mcp/tools/` and registered in `mod.rs`.

## Relevant Files
- `crates/cclab-genesis/src/services/platform_sync/github.rs`: Needs a new `fetch_issues` method.
- `crates/cclab-genesis/src/services/platform_sync/mod.rs`: Needs to expose `fetch_issues` in `PlatformSyncService`.
- `crates/cclab-genesis/src/services/platform_sync/types.rs`: Needs new types: `FetchedIssue`, `FetchIssuesOptions`, `IssueDag`, `DagEdge`.
- `crates/cclab-genesis/src/models/frontmatter.rs`: Needs to update `State` struct to include the `dag` field.
- `crates/cclab-genesis/src/mcp/tools/platform_sync.rs`: Good place to add the `genesis_fetch_issues` tool definition and execution logic.

## Impact Analysis
- **Data Model**: Adding `dag` to `State` is a significant but backwards-compatible change (optional field).
- **Workflows**: `genesis_run_change` will need to be updated later to utilize the `dag` section for routing through multiple issues.
- **CLI**: The new tool will be available via MCP, improving the ability to start changes from existing issues.

## Technical Considerations
- **GitHub API**: Fetching sub-issues and relationships requires the `GraphQL-Features: sub_issues` header and specific GraphQL queries.
- **Cycle Detection**: Implementing Kahn's algorithm or similar for DAG validation and topological sorting.
- **Artifact Generation**: Writing `issue_{NNN}.md` and `dependency_graph.md` files as specified.

## Spec Recommendations
- Extend `cclab-genesis/fetch-issues` spec if any implementation details deviate during development.
- Ensure `STATE.yaml` schema version is bumped or handles the new field gracefully.

