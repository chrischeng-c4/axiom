---
id: genesis-aurora-fixes
type: proposal
version: 1
created_at: 2026-02-03T02:29:32.020557+00:00
updated_at: 2026-02-03T02:29:32.020557+00:00
author: mcp
status: proposed
iteration: 1
summary: "Fix Mermaid+ generator formatting, remove Bash permission from impl-change, and align specs to Mermaid+-only guidance."
history:
  - timestamp: 2026-02-03T02:29:32.020557+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-02-03T02:29:37.395941+00:00
    agent: "codex:deep"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-02-03T02:29:47.877144+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 13
  new_files: 0---

<proposal>

# Change: genesis-aurora-fixes

## Summary

Fix Mermaid+ generator formatting, remove Bash permission from impl-change, and align specs to Mermaid+-only guidance.

## Why

Mermaid+ generators currently place YAML frontmatter outside the mermaid code block, which breaks the Mermaid+ format and downstream rendering/parsing expectations. The impl-change workflow grants Bash permission, adding avoidable risk and exceeding the minimal tool set needed. Several specs still describe XState-based Mermaid+ behavior that is not implemented, creating mismatched guidance. This change aligns generator output, permissions, and documentation with the current, supported Mermaid+ format.

## What Changes

- Embed Mermaid+ YAML frontmatter inside the ```mermaid``` code block for all Aurora “plus” generators while preserving validation warnings.
- Remove Bash from the allowed tool list in the impl-change Claude orchestration path so only MCP tools are permitted.
- Update Mermaid+ and planning specs to remove XState references and describe the supported Mermaid+ format (YAML metadata + Mermaid diagram).

## Impact

- **Scope**: minor
- **Affected Files**: ~13
- **New Files**: ~0
- Affected code: `crates/cclab-aurora/src/diagrams/flowchart_plus/generator.rs`, `crates/cclab-aurora/src/diagrams/sequence_plus/generator.rs`, `crates/cclab-aurora/src/diagrams/class_plus/generator.rs`, `crates/cclab-aurora/src/diagrams/erd_plus/generator.rs`, `crates/cclab-aurora/src/diagrams/requirement_plus/generator.rs`, `crates/cclab-aurora/src/diagrams/mindmap_plus/generator.rs`, `crates/cclab-aurora/src/diagrams/journey_plus/generator.rs`, `crates/cclab-aurora/src/diagrams/mermaid_plus/generator.rs`, `crates/cclab-aurora/src/diagrams/state_plus/generator.rs`, `crates/cclab-genesis/src/orchestrator/claude.rs`, `cclab/specs/cclab-genesis/plan-change.md`, `cclab/specs/cclab-aurora/mermaid-plus-format.md`, `cclab/specs/cclab-aurora/mermaid-plus-conversion.md`
- **Breaking Changes**: None expected. The Mermaid+ output change is a formatting correction that should improve compatibility.

</proposal>
