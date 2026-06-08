---
id: genesis-agent-272-273
type: proposal
version: 2
created_at: 2026-02-12T11:34:23.106441+00:00
updated_at: 2026-02-12T11:34:23.106441+00:00
iteration: 1
scope: minor
spec_plan:
  - id: delegate-agent-impl
    title: "Implement delegate-agent spec: rename, action routing, artifact response"
    depends: []
    context_refs:
      codebase: ["agent.rs", "mod.rs", "helpers.rs", "config.rs", "cli_mapper.rs"]
      spec: ["cclab-genesis/delegate-agent", "cclab-genesis/delegate-agent-coverage", "genesis/prompt-registry"]
      knowledge: ["40-mcp/index.md"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 1 }
      - { source: gap_codebase_spec, gap_index: 2 }
      - { source: gap_codebase_spec, gap_index: 3 }
      - { source: gap_codebase_spec, gap_index: 5 }
      - { source: gap_codebase_knowledge, gap_index: 1 }
    affected_code: ["crates/cclab-genesis/src/mcp/tools/agent.rs", "crates/cclab-genesis/src/mcp/tools/mod.rs", "crates/cclab-genesis/src/mcp/tools/run_change/helpers.rs", "crates/cclab-genesis/src/mcp/config.rs", "crates/cclab-genesis/src/orchestrator/cli_mapper.rs", ".claude/skills/cclab-genesis-agent/SKILL.md"]
  - id: delegate-agent-recovery
    title: "Error recovery: retry + fallback chain for delegate-agent"
    depends: [delegate-agent-impl]
    context_refs:
      spec: ["cclab-genesis/delegate-agent Error Recovery section"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 4 }
    affected_code: ["crates/cclab-genesis/src/mcp/tools/agent.rs"]
history:
  - timestamp: 2026-02-12T11:34:23.106441+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
---

<proposal>

# Spec Navigation Map: genesis-agent-272-273

## Scope Overview (Mindmap)

```mermaid
mindmap
  root((genesis-agent-272-273))  
    Rename
      genesis_agent to genesis_delegate_agent
      MCP tool definition
      helpers.rs constant
      config.rs excludeTools
      cli_mapper.rs DisallowedMcpTools
      Skill SKILL.md
    Action Routing
      Expand enum from 3 to 30+
      Per-action prompt templates
      custom fallback for unknown actions
      Action to artifact mapping
    Response Format
      Remove raw stdout
      Return status/verification/usage/next
      Match genesis MCP convention
      Log raw output to server log
    Error Recovery
      Retry once on transient failure
      Fallback to next agent in chain
      Partial progress handling
      Mainthread fallback
```

## Spec Dependency Graph (Block Diagram)

```mermaid
block-beta
  columns 3

  delegate_agent_impl["delegate-agent-impl\n codebase: agent.rs, mod.rs, helpers.rs, config.rs, cli_mapper.rs\n gaps: codebase_spec#1, codebase_spec#2, codebase_spec#3, codebase_spec#5, codebase_knowledge#1"]
  delegate_agent_recovery["delegate-agent-recovery\n gaps: codebase_spec#4"]

  delegate_agent_impl --> delegate_agent_recovery
```

## Spec Execution Order

1. **delegate-agent-impl** — Implement delegate-agent spec: rename, action routing, artifact response
   - code: crates/cclab-genesis/src/mcp/tools/agent.rs, crates/cclab-genesis/src/mcp/tools/mod.rs, crates/cclab-genesis/src/mcp/tools/run_change/helpers.rs, crates/cclab-genesis/src/mcp/config.rs, crates/cclab-genesis/src/orchestrator/cli_mapper.rs, .claude/skills/cclab-genesis-agent/SKILL.md
2. **delegate-agent-recovery** — Error recovery: retry + fallback chain for delegate-agent
   - depends: delegate-agent-impl
   - code: crates/cclab-genesis/src/mcp/tools/agent.rs

</proposal>
