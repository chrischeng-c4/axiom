---
id: nova-analyst-agent
type: proposal
version: 1
created_at: 2026-01-31T09:53:23.304575+00:00
updated_at: 2026-01-31T09:53:23.304575+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add AnalystAgent to cclab-nova for requirements analysis with composable integrations"
history:
  - timestamp: 2026-01-31T09:53:23.304575+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 8
  new_files: 10
affected_specs:
  - id: analyst-agent
    path: specs/analyst-agent.md
    depends: []
  - id: analysis-tools
    path: specs/analysis-tools.md
    depends: [analyst-agent]
  - id: storage-backend
    path: specs/storage-backend.md
    depends: [analyst-agent]
  - id: platform-integrations
    path: specs/platform-integrations.md
    depends: [analyst-agent]
---

<proposal>

# Change: nova-analyst-agent

## Summary

Add AnalystAgent to cclab-nova for requirements analysis with composable integrations

## Why

Before writing code, developers need to clarify requirements and research solutions. Currently cclab-nova only has CodingAgent for implementation tasks. Adding AnalystAgent enables a complete development workflow: analysis → specification → implementation → review. The agent should support composable integrations (GitHub, GitLab, Jira) to read issues and context from existing project management tools, with pluggable storage for session persistence.

## What Changes

- Add AnalystAgent struct with builder pattern in src/agents/analyst.rs
- Add new analysis tools: AskUserTool, TakeNoteTool, WebSearchTool, WebFetchTool
- Add storage trait and implementations (MemoryStorage, FileStorage) for session persistence
- Add platform integrations: GitHubIntegration, GitLabIntegration, JiraIntegration
- Refactor agent module structure: move CodingAgent to src/agents/coding.rs
- Create shared Agent trait for common interface
- Update lib.rs exports and add convenience functions nova::analyst(), nova::coder()

## Impact

- **Scope**: minor
- **Affected Files**: ~8
- **New Files**: ~10
- Affected specs:
  - `analyst-agent` (no dependencies)
  - `analysis-tools` → depends on: `analyst-agent`
  - `storage-backend` → depends on: `analyst-agent`
  - `platform-integrations` → depends on: `analyst-agent`
- Affected code: `crates/cclab-nova/src/lib.rs`, `crates/cclab-nova/src/agent.rs`

</proposal>
