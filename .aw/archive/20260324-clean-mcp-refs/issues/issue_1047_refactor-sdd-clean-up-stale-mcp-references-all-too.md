---
number: 1047
title: "refactor(sdd): clean up stale MCP references — all tools are CLI now"
state: open
labels: [priority:p2, crate:sdd, type:refactor]
group: "mcp-cleanup"
---

# #1047 — refactor(sdd): clean up stale MCP references — all tools are CLI now

## Problem

SDD has fully migrated from MCP server to CLI execution. `skills/run-change.md` explicitly states:

> **CLI commands only. Do NOT use MCP tool calls.**

And `sdd-cli.md` documents the migration intent:

> The SDD workflow currently relies heavily on an MCP server for execution. To reduce dependency on the MCP server and provide a direct, standalone interaction model...

However, **dozens of spec files still reference MCP** — stale terminology, `mcp/tools/*.rs` file paths, "MCP tool" descriptions, and MCP config templates. This creates confusion about the actual architecture.

## Scope

### Stale `files:` frontmatter referencing `mcp/tools/`

These specs list `mcp/tools/*.rs` source files that may have been moved/renamed to CLI paths:

| Spec | Stale `files:` entry |
|------|---------------------|
| `logic/state-machine.md` | `mcp/tools/phase_transition.rs` |
| `logic/executor-resolution.md` | `mcp/tools/workflow_common.rs` |
| `logic/implement-task.md` | `mcp/tools/change_impl/{common,create,review,revise}.rs` |
| `logic/post-clarifications.md` | `mcp/tools/post_clarifications/{mod,create}.rs` |
| `logic/pre-clarifications.md` | `mcp/tools/create_pre_clarifications.rs` |
| `logic/change-merge.md` | `mcp/tools/change_merge/create.rs` |
| `logic/change-spec.md` | `mcp/tools/change_spec/{common,create,review,revise}.rs` |
| `logic/restructure-input.md` | `mcp/tools/restructure_input.rs` |
| `tools/utils/delegate-agent.md` | `mcp/tools/agent.rs` |
| `tools/utils/write-artifact.md` | `mcp/tools/artifact_write.rs` |
| `tools/utils/read-artifact.md` | `mcp/tools/artifact_read.rs` |
| `tools/utils/fetch-issues.md` | `mcp/tools/fetch_issues.rs` |
| `tools/utils/read-implementation-summary.md` | `mcp/tools/implementation.rs` |
| `tools/utils/analyze-code-for-spec.md` | `mcp/tools/analyze/{mod,python,typescript,rust_lang,suggestions}.rs` |
| `tools/utils/platform-sync.md` | `mcp/tools/platform_sync.rs` |
| `tools/utils/validate-spec-completeness.md` | `mcp/tools/validate_spec.rs` |
| `tools/utils/validate-change.md` | `mcp/tools/validate.rs` |

### Stale "MCP tool" terminology

| Spec | Line | Content |
|------|------|---------|
| `interfaces/tools/artifact-tools.md` | title | "MCP Artifact Tools — OpenRPC Definitions" |
| `interfaces/tools/utility-tools.md` | title | "MCP Utility Tools — OpenRPC Definitions" |
| `tools/utils/write-artifact.md` | desc | "Unified MCP tool for all artifact lifecycle operations" |
| `tools/utils/read-implementation-summary.md` | desc | "MCP tool that produces a markdown-formatted implementation summary" |
| `tools/utils/list-changed-files.md` | desc | "MCP tool that lists files changed between branches" |
| `tools/utils/validate-change.md` | multiple | "direct MCP tool invocation", "MCP response" |
| `tools/utils/delegate-agent.md` | L12 | "via MCP tool" |
| `tools/utils/delegate-agent.md` | L184-185 | `mcp__cclab-mcp__sdd_delegate_agent` disallow pattern |
| `skills/agent.md` | multiple | "MCP tool", "unified MCP tool" |
| `skills/run-change.md` | L18 | "calling the `sdd_run_change` MCP tool" |
| `skills/fillback.md` | L104 | "Uses Lens MCP tools for code analysis" |
| `generate/template-claude-md.md` | multiple | "MCP tool pointers", `mcp__cclab__sdd_*` |
| `generate/template-knowledge-index.md` | L66 | "LLM tools can read knowledge via MCP" |
| `generate/template-mcp-configs.md` | entire file | MCP config templates — may be fully obsolete |

### Possibly obsolete spec

`generate/template-mcp-configs.md` — entire file documents MCP config generation for `.mcp.json`, `.gemini/settings.json`, `.codex/config.toml`. If MCP server is no longer used, this spec may be fully obsolete.

## Acceptance Criteria

- [ ] All `files:` frontmatter updated to reflect actual source file paths (post CLI migration)
- [ ] "MCP tool" terminology replaced with "CLI command" or "tool" as appropriate
- [ ] `mcp__cclab-mcp__*` patterns in disallowedTools updated or removed
- [ ] `generate/template-mcp-configs.md` evaluated — archive if obsolete
- [ ] `sdd-cli.md` R5 "Parity with MCP Execution" reworded (MCP is gone, CLI is the only path)
- [ ] No remaining references imply MCP server is in active use
