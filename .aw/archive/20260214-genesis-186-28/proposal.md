---
id: genesis-186-28
type: proposal
version: 2
created_at: 2026-02-14T03:46:15.125849+00:00
updated_at: 2026-02-14T03:46:15.125849+00:00
iteration: 4
scope: minor
spec_plan:
  - id: code-analysis-service-v2
    title: "Agnostic Code Analysis and LLM Enrichment Service"
    depends: []
    context_refs:
      codebase: ["mcp/tools/analyze.rs"]
      knowledge: ["30-claude/skills.md"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 0 }
    affected_code: ["crates/cclab-genesis/src/mcp/tools/analyze.rs"]
  - id: diagram-auto-generation
    title: "AST-to-Aurora Structured Diagram Mapping"
    depends: [code-analysis-service-v2]
    context_refs:
      codebase: ["mcp/tools/analyze.rs", "spec_service.rs"]
      spec: ["create-spec"]
    affected_code: ["crates/cclab-genesis/src/mcp/tools/analyze.rs", "crates/cclab-genesis/src/services/spec_service.rs"]
  - id: spec-validation-integration
    title: "Spec Completeness Validator and Aurora Integration"
    depends: [diagram-auto-generation]
    context_refs:
      codebase: ["spec_service.rs"]
      spec: ["create-spec"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 4 }
    affected_code: ["crates/cclab-genesis/src/services/spec_service.rs", "crates/cclab-genesis/src/mcp/tools/spec.rs"]
  - id: workflow-enforcement-repair
    title: "Unified Workflow Verdicts and Escalation Enforcement"
    depends: []
    context_refs:
      codebase: ["run_change/helpers.rs", "run_change/spec.rs"]
      spec: ["review-spec", "verdict-unification"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 1 }
      - { source: gap_codebase_spec, gap_index: 2 }
    affected_code: ["crates/cclab-genesis/src/mcp/tools/run_change/helpers.rs", "crates/cclab-genesis/src/mcp/tools/run_change/spec.rs", "crates/cclab-genesis/src/mcp/tools/run_change/proposal.rs"]
history:
  - timestamp: 2026-02-14T03:46:15.125849+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
---

<proposal>

# Spec Navigation Map: genesis-186-28

## Scope Overview (Mindmap)

```mermaid
mindmap
  root((genesis-186-28))  
    Spec-Driven Development Core
      LLM Enrichment v2 (Requirements/Scenarios)
      --quick fast-path (AST-only) support
      AST-to-Aurora diagram mapping
    Workflow & Quality
      Strict revision thresholds (4-rejection limit)
      Mainthread escalation logic (mainthread_must_fix)
      Unified verdict models
    Validation Integration
      Spec completeness validator integration
      Aurora structured input generation
```

## Spec Dependency Graph (Block Diagram)

```mermaid
block-beta
  columns 3

  code_analysis_service_v2["code-analysis-service-v2\n codebase: mcp/tools/analyze.rs\n gaps: codebase_spec#0"]
  diagram_auto_generation["diagram-auto-generation\n codebase: mcp/tools/analyze.rs, spec_service.rs"]
  spec_validation_integration["spec-validation-integration\n codebase: spec_service.rs\n gaps: codebase_spec#4"]
  workflow_enforcement_repair["workflow-enforcement-repair\n codebase: run_change/helpers.rs, run_change/spec.rs\n gaps: codebase_spec#1, codebase_spec#2"]

  code_analysis_service_v2 --> diagram_auto_generation
  diagram_auto_generation --> spec_validation_integration
```

## Spec Execution Order

1. **code-analysis-service-v2** — Agnostic Code Analysis and LLM Enrichment Service
   - code: crates/cclab-genesis/src/mcp/tools/analyze.rs
2. **diagram-auto-generation** — AST-to-Aurora Structured Diagram Mapping
   - depends: code-analysis-service-v2
   - code: crates/cclab-genesis/src/mcp/tools/analyze.rs, crates/cclab-genesis/src/services/spec_service.rs
3. **spec-validation-integration** — Spec Completeness Validator and Aurora Integration
   - depends: diagram-auto-generation
   - code: crates/cclab-genesis/src/services/spec_service.rs, crates/cclab-genesis/src/mcp/tools/spec.rs
4. **workflow-enforcement-repair** — Unified Workflow Verdicts and Escalation Enforcement
   - code: crates/cclab-genesis/src/mcp/tools/run_change/helpers.rs, crates/cclab-genesis/src/mcp/tools/run_change/spec.rs, crates/cclab-genesis/src/mcp/tools/run_change/proposal.rs

</proposal>
