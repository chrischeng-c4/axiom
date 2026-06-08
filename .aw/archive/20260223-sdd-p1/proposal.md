---
id: sdd-p1
type: proposal
version: 2
created_at: 2026-02-23T14:28:36.266183+00:00
updated_at: 2026-02-23T14:28:36.266183+00:00
iteration: 1
scope: patch
spec_plan:
  - id: sdd-p1-dag-context-loop
    title: "Fix DAG Context Loop Complexity Routing (#471)"
    depends: []
    context_refs:
      codebase: ["crates/cclab-sdd/src/mcp/tools/run_change/dag_loop.rs"]
      spec: ["cclab/specs/cclab-sdd/tools/run-change/dag-loop.md"]
      knowledge: ["cclab/changes/sdd-p1/gap_codebase_spec.md"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 6 }
    affected_code: ["crates/cclab-sdd/src/mcp/tools/run_change/dag_loop.rs"]
  - id: sdd-p1-review-prompts
    title: "Standardize Verdict Labels and Review Checklists (#467, #469, #470)"
    depends: []
    context_refs:
      codebase: ["crates/cclab-sdd/src/mcp/tools/run_change/prompts/"]
      knowledge: ["cclab/changes/sdd-p1/gap_codebase_spec.md", "cclab/changes/sdd-p1/context_clarifications.md"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 7 }
      - { source: gap_codebase_spec, gap_index: 8 }
    affected_code: ["crates/cclab-sdd/src/mcp/tools/run_change/prompts/"]
  - id: sdd-p1-revise-actions
    title: "Fix Revise Actions Using Create (#468)"
    depends: []
    context_refs:
      codebase: ["crates/cclab-sdd/src/mcp/tools/run_change/clarify.rs", "crates/cclab-sdd/src/mcp/tools/run_change/proposal.rs"]
      knowledge: ["cclab/changes/sdd-p1/context_clarifications.md"]
    affected_code: ["crates/cclab-sdd/src/mcp/tools/run_change/clarify.rs", "crates/cclab-sdd/src/mcp/tools/run_change/proposal.rs"]
history:
  - timestamp: 2026-02-23T14:28:36.266183+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
---

<proposal>

# Spec Navigation Map: sdd-p1

## Scope Overview (Mindmap)

```mermaid
mindmap
  root((sdd-p1))  
    Logic Fixes
      DAG Loop Routing
      Revise Action Verbs
    Prompt Consistency
      Verdict Labels
      Review Checklists
      Gap Analysis Fields
```

## Spec Dependency Graph (Block Diagram)

```mermaid
block-beta
  columns 3

  sdd_p1_dag_context_loop["sdd-p1-dag-context-loop\n codebase: crates/cclab-sdd/src/mcp/tools/run_change/dag_loop.rs\n gaps: codebase_spec#6"]
  sdd_p1_review_prompts["sdd-p1-review-prompts\n codebase: crates/cclab-sdd/src/mcp/tools/run_change/prompts/\n gaps: codebase_spec#7, codebase_spec#8"]
  sdd_p1_revise_actions["sdd-p1-revise-actions\n codebase: crates/cclab-sdd/src/mcp/tools/run_change/clarify.rs, crates/cclab-sdd/src/mcp/tools/run_change/proposal.rs"]

```

## Spec Execution Order

1. **sdd-p1-dag-context-loop** — Fix DAG Context Loop Complexity Routing (#471)
   - code: crates/cclab-sdd/src/mcp/tools/run_change/dag_loop.rs
2. **sdd-p1-review-prompts** — Standardize Verdict Labels and Review Checklists (#467, #469, #470)
   - code: crates/cclab-sdd/src/mcp/tools/run_change/prompts/
3. **sdd-p1-revise-actions** — Fix Revise Actions Using Create (#468)
   - code: crates/cclab-sdd/src/mcp/tools/run_change/clarify.rs, crates/cclab-sdd/src/mcp/tools/run_change/proposal.rs

</proposal>
