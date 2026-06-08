---
id: nova-async-clarification
type: proposal
version: 1
created_at: 2026-02-01T10:21:21.861037+00:00
updated_at: 2026-02-01T10:21:21.861037+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add async clarification workflow to AnalystAgent with platform comment support and session persistence"
history:
  - timestamp: 2026-02-01T10:21:21.861037+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-02-01T10:21:34.986473+00:00
    agent: "unknown"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-02-01T10:21:48.162808+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
  - timestamp: 2026-02-01T10:26:17.330035+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-02-01T10:26:36.205796+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 8
  new_files: 0
affected_specs:
  - id: analyst-agent-async
    path: specs/analyst-agent-async.md
    depends: []
  - id: platform-commenting
    path: specs/platform-commenting.md
    depends: []
  - id: clarification-tools
    path: specs/clarification-tools.md
    depends: []---

<proposal>

# Change: nova-async-clarification

## Summary

Add async clarification workflow to AnalystAgent with platform comment support and session persistence

## Why

Requirements analysis often requires asynchronous input from stakeholders. Currently, AnalystAgent only supports synchronous clarification. This change enables agents to ask questions on platforms like GitHub or Jira, pause execution, and resume once stakeholders respond, while maintaining full conversation context.

## What Changes

- Add 'messages' to SessionState for full LLM history persistence
- Add 'post_comment' to PlatformIntegration trait and implementations (GitHub, GitLab, Jira)
- Create 'post_comment' tools that support markdown checkboxes and trigger session pause
- Implement markdown comment parsing for checkbox selections and reply text
- Enhance AnalystAgent to support session resume by fetching and parsing new comments

## Impact

- **Scope**: minor
- **Affected Files**: ~8
- **New Files**: ~0
- Affected specs:
  - `analyst-agent-async` (no dependencies)
  - `platform-commenting` (no dependencies)
  - `clarification-tools` (no dependencies)
- Affected code: `crates/cclab-nova/src/storage/mod.rs`, `crates/cclab-nova/src/integrations/mod.rs`, `crates/cclab-nova/src/integrations/github.rs`, `crates/cclab-nova/src/integrations/gitlab.rs`, `crates/cclab-nova/src/integrations/jira.rs`, `crates/cclab-nova/src/agents/analyst.rs`, `crates/cclab-nova/src/tools/analysis.rs`

</proposal>
