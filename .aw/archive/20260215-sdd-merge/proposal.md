---
id: sdd-merge
type: proposal
version: 2
created_at: 2026-02-15T03:45:30.173090+00:00
updated_at: 2026-02-15T03:45:30.173090+00:00
iteration: 1
scope: major
spec_plan:
  - id: crate-unification
    title: "Crate Unification and Rename"
    depends: []
    context_refs:
      codebase: ["G3: Unimplemented Crate Unification", "G5: Aurora Relay Removal"]
      spec: ["migration-architecture"]
      knowledge: ["Clarification Q1: Rename scope"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 3 }
      - { source: gap_codebase_spec, gap_index: 5 }
    affected_code: ["crates/cclab-genesis", "crates/cclab-aurora", "Cargo.toml"]
  - id: mcp-router-unification
    title: "Unified MCP Router and Registry"
    depends: [crate-unification]
    context_refs:
      codebase: ["G1: Redundant Registry Implementations", "G2: Missing PDG Tools in Unified Router", "G4: Duplicate HTTP Server Logic", "G6: Redundant CLI Server Commands"]
      spec: ["genesis-codegen-orchestration"]
      knowledge: ["40-mcp/dynamic-config.md"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 1 }
      - { source: gap_codebase_spec, gap_index: 2 }
      - { source: gap_codebase_spec, gap_index: 4 }
      - { source: gap_codebase_spec, gap_index: 6 }
      - { source: gap_spec_knowledge, gap_index: 3 }
    affected_code: ["crates/cclab-server/src/mcp", "crates/cclab-sdd/src/mcp"]
  - id: generator-decoupling
    title: "Generator Decoupling and Legacy Removal"
    depends: [mcp-router-unification]
    context_refs:
      codebase: ["G5: Aurora Relay Removal"]
      spec: ["aurora-codegen-system"]
      knowledge: ["spec-to-code/index.md"]
    gap_repairs:
      - { source: gap_spec_knowledge, gap_index: 2 }
    affected_code: ["crates/cclab-sdd/src/fillback", "crates/cclab-sdd/src/mcp/tools"]
  - id: manifest-handling
    title: "Manifest Handling in Merge Logic"
    depends: [crate-unification]
    context_refs:
      spec: ["merge-change"]
      knowledge: ["genesis-372-impact.md"]
    gap_repairs:
      - { source: gap_spec_knowledge, gap_index: 1 }
    affected_code: ["crates/cclab-sdd/src/cli/archive.rs"]
  - id: review-verdict-unification
    title: "Unified Review Verdicts"
    depends: [crate-unification]
    context_refs:
      knowledge: ["verdict-unification.md"]
    gap_repairs:
      - { source: gap_spec_knowledge, gap_index: 4 }
    affected_code: ["crates/cclab-sdd/src/models/review.rs"]
  - id: prompt-template-update
    title: "Prompt Template Updates"
    depends: [crate-unification]
    context_refs:
      knowledge: ["Clarification Q2: MCP tool prefix"]
    gap_repairs:
      - { source: gap_spec_knowledge, gap_index: 5 }
    affected_code: ["crates/cclab-sdd/src/orchestrator/prompts.rs", "docs/PROMPT_TEMPLATE_INTEGRATION.md"]
history:
  - timestamp: 2026-02-15T03:45:30.173090+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
---

<proposal>

# Spec Navigation Map: sdd-merge

## Scope Overview (Mindmap)

```mermaid
mindmap
  root((sdd-merge))  
    Core Architecture
      Crate Rename
      Dependency Updates
      CLI Command Unification
    MCP Integration
      Unified Router
      Tool Registry
      Tool Renaming
      PDG Tools
    Generator Logic
      Decoupling
      Relay Removal
      Manifest Handling
    Workflow Standards
      Review Verdicts
      Prompt Templates
      Stage-based Filtering
```

## Spec Dependency Graph (Block Diagram)

```mermaid
block-beta
  columns 3

  crate_unification["crate-unification\n codebase: G3: Unimplemented Crate Unification, G5: Aurora Relay Removal\n gaps: codebase_spec#3, codebase_spec#5"]
  mcp_router_unification["mcp-router-unification\n codebase: G1: Redundant Registry Implementations, G2: Missing PDG Tools in Unified Router, G4: Duplicate HTTP Server Logic, G6: Redundant CLI Server Commands\n gaps: codebase_spec#1, codebase_spec#2, codebase_spec#4, codebase_spec#6, spec_knowledge#3"]
  generator_decoupling["generator-decoupling\n codebase: G5: Aurora Relay Removal\n gaps: spec_knowledge#2"]
  manifest_handling["manifest-handling\n gaps: spec_knowledge#1"]
  review_verdict_unification["review-verdict-unification\n gaps: spec_knowledge#4"]
  prompt_template_update["prompt-template-update\n gaps: spec_knowledge#5"]

  crate_unification --> mcp_router_unification
  mcp_router_unification --> generator_decoupling
  crate_unification --> manifest_handling
  crate_unification --> review_verdict_unification
  crate_unification --> prompt_template_update
```

## Spec Execution Order

1. **crate-unification** — Crate Unification and Rename
   - code: crates/cclab-genesis, crates/cclab-aurora, Cargo.toml
2. **manifest-handling** — Manifest Handling in Merge Logic
   - depends: crate-unification
   - code: crates/cclab-sdd/src/cli/archive.rs
3. **mcp-router-unification** — Unified MCP Router and Registry
   - depends: crate-unification
   - code: crates/cclab-server/src/mcp, crates/cclab-sdd/src/mcp
4. **generator-decoupling** — Generator Decoupling and Legacy Removal
   - depends: mcp-router-unification
   - code: crates/cclab-sdd/src/fillback, crates/cclab-sdd/src/mcp/tools
5. **prompt-template-update** — Prompt Template Updates
   - depends: crate-unification
   - code: crates/cclab-sdd/src/orchestrator/prompts.rs, docs/PROMPT_TEMPLATE_INTEGRATION.md
6. **review-verdict-unification** — Unified Review Verdicts
   - depends: crate-unification
   - code: crates/cclab-sdd/src/models/review.rs

</proposal>
