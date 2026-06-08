---
id: prism-pdg
type: proposal
version: 1
created_at: 2026-01-31T02:57:18.223743+00:00
updated_at: 2026-01-31T02:57:18.223743+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add statement-level Program Dependence Graph (PDG) to Prism for Python code intelligence."
history:
  - timestamp: 2026-01-31T02:57:18.223743+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-31T02:59:26.080657+00:00
    agent: "unknown"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-31T02:59:45.008378+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
  - timestamp: 2026-01-31T03:04:43.780361+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-01-31T03:05:01.488198+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 12
  new_files: 6---

<proposal>

# Change: prism-pdg

## Summary

Add statement-level Program Dependence Graph (PDG) to Prism for Python code intelligence.

## Why

To provide LLMs and developers with deep code intelligence, enabling advanced features like precise program slicing, change impact analysis, security taint tracking, and dead code detection. This significantly improves the quality of code understanding and automated refactoring.

## What Changes

- Implement statement-level Control Flow Graph (CFG) for Python including branches and exceptions.
- Implement Post-Dominator Tree construction for control dependency analysis.
- Implement Reaching Definitions and Def-Use chain tracking for data dependencies.
- Implement forward and backward program slicing based on the PDG.
- Add Impact Analysis for change detection using forward slicing.
- Add Taint Tracking for security analysis by tracing data flow from sources to sinks.
- Add Dead Code Detection for statements not contributing to program output or unreachable in CFG.
- Support cross-file inter-procedural analysis via call graph integration.
- Expose PDG capabilities via new MCP tools (prism_pdg, prism_slice, prism_impact, prism_taint).

## Impact

- **Scope**: minor
- **Affected Files**: ~12
- **New Files**: ~6
- Affected code: `crates/cclab-prism/src/semantic/pdg/mod.rs`, `crates/cclab-prism/src/semantic/pdg/cfg.rs`, `crates/cclab-prism/src/semantic/pdg/data_flow.rs`, `crates/cclab-prism/src/semantic/pdg/dominator.rs`, `crates/cclab-prism/src/semantic/mod.rs`, `crates/cclab-prism/src/mcp/tools.rs`, `crates/cclab-prism/src/server/daemon.rs`

</proposal>
