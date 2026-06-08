---
id: cli-to-mcp
type: proposal
version: 1
created_at: 2026-02-05T09:01:39.385102+00:00
updated_at: 2026-02-05T09:01:39.385102+00:00
author: mcp
status: proposed
iteration: 1
summary: "Migrate plan-change orchestration to a state-aware MCP tool (genesis_plan_change) and keep CLI as a thin wrapper"
history:
  - timestamp: 2026-02-05T09:01:39.385102+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-02-05T09:03:57.541030+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-02-05T09:04:08.904955+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 7
  new_files: 1
affected_specs:
  - id: plan-change
    path: specs/plan-change.md
    depends: []---

<proposal>

# Change: cli-to-mcp

## Summary

Migrate plan-change orchestration to a state-aware MCP tool (genesis_plan_change) and keep CLI as a thin wrapper

## Why

The current plan-change workflow is CLI-driven, which forces mainthread to spawn subprocesses and makes state-aware orchestration indirect. A dedicated MCP tool can inspect STATE.yaml and existing artifacts to return the next action and instructions, mirroring the decide-change dispatcher pattern. This keeps planning logic consistent across workflows, enables tool-only execution paths, and reduces fragility around CLI invocation while preserving idempotent plan-change behavior.

## What Changes

- Add a new genesis_plan_change MCP tool that inspects STATE.yaml plus proposal/spec/task/review artifacts and returns the next action with structured instructions.
- Integrate the tool into the MCP registry and OpenRPC specs, and add tests to cover action routing across plan-change phases.
- Update plan-change skill/docs and CLI entrypoint to call the MCP tool (or act as a wrapper) while keeping backward compatibility.

## Impact

- **Scope**: minor
- **Affected Files**: ~7
- **New Files**: ~1
- Affected specs:
  - `plan-change` (no dependencies)
- Affected code: `crates/cclab-genesis/src/mcp/tools/plan_change.rs`, `crates/cclab-genesis/src/mcp/tools/mod.rs`, `cclab/specs/cclab-genesis/plan-change/api.openrpc.json`, `crates/cclab-genesis/src/cli/plan.rs`, `crates/cclab-genesis/templates/mainthread/skills/cclab-genesis-plan-change/SKILL.md`, `cclab/specs/cclab-genesis/plan-change/README.md`
- **Breaking Changes**: No

</proposal>
