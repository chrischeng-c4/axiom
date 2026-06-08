---
id: genesis-295-302
type: proposal
version: 2
created_at: 2026-02-13T08:00:27.508147+00:00
updated_at: 2026-02-13T08:00:27.508147+00:00
iteration: 1
scope: major
spec_plan:
  - id: remove-legacy-v1
    title: "Remove legacy v1 proposal and schema paths"
    depends: []
    affected_code: ["crates/cclab-genesis/src/services/proposal_service.rs", "crates/cclab-genesis/src/mcp/tools/run_change/helpers.rs"]
  - id: openrpc-schema-sync
    title: "Sync OpenRPC response schemas with actual code return fields"
    depends: [remove-legacy-v1]
    affected_code: ["crates/cclab-genesis/src/mcp/tools/run_change/mod.rs", "crates/cclab-genesis/src/mcp/tools/run_change/helpers.rs"]
  - id: run-change-issues-param
    title: "Add issues param to run_change, eliminate fetch_issues action"
    depends: [openrpc-schema-sync]
    affected_code: ["crates/cclab-genesis/src/mcp/tools/run_change/mod.rs", "crates/cclab-genesis/src/mcp/tools/fetch_issues.rs"]
  - id: tag-union-validation
    title: "Refactor spec validation to compositional tag-union logic"
    depends: []
    affected_code: ["crates/cclab-genesis/src/services/spec_service.rs", "crates/cclab-genesis/src/models/spec_rules.rs"]
  - id: plus-diagrams-specs
    title: "Add Class+, Sequence+, Flowchart+, Requirement+ diagrams to genesis specs"
    depends: [tag-union-validation, openrpc-schema-sync]
    affected_code: ["cclab/specs/cclab-genesis/create-proposal.md", "cclab/specs/cclab-genesis/create-spec.md", "cclab/specs/cclab-genesis/run-change/README.md", "cclab/specs/cclab-genesis/implement-change.md"]
history:
  - timestamp: 2026-02-13T08:00:27.508147+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
---

<proposal>

# Spec Navigation Map: genesis-295-302

## Scope Overview (Mindmap)

```mermaid
mindmap
  root((genesis-295-302))  
    Validation
      tag-union refactor
      auto-tag alignment
      acceptance criteria
    Legacy Removal
      v1 proposal path
      v1 schema dispatch
      v1 frontmatter format
    OpenRPC Schemas
      response field audit
      issues param
      fetch_issues elimination
    Spec Enrichment
      Class Plus diagrams
      Sequence Plus diagrams
      Flowchart Plus diagrams
      Requirement Plus diagrams
```

## Spec Dependency Graph (Block Diagram)

```mermaid
block-beta
  columns 3

  remove_legacy_v1["remove-legacy-v1"]
  openrpc_schema_sync["openrpc-schema-sync"]
  run_change_issues_param["run-change-issues-param"]
  tag_union_validation["tag-union-validation"]
  plus_diagrams_specs["plus-diagrams-specs"]

  remove_legacy_v1 --> openrpc_schema_sync
  openrpc_schema_sync --> run_change_issues_param
  tag_union_validation --> plus_diagrams_specs
  openrpc_schema_sync --> plus_diagrams_specs
```

## Spec Execution Order

1. **remove-legacy-v1** — Remove legacy v1 proposal and schema paths
   - code: crates/cclab-genesis/src/services/proposal_service.rs, crates/cclab-genesis/src/mcp/tools/run_change/helpers.rs
2. **openrpc-schema-sync** — Sync OpenRPC response schemas with actual code return fields
   - depends: remove-legacy-v1
   - code: crates/cclab-genesis/src/mcp/tools/run_change/mod.rs, crates/cclab-genesis/src/mcp/tools/run_change/helpers.rs
3. **run-change-issues-param** — Add issues param to run_change, eliminate fetch_issues action
   - depends: openrpc-schema-sync
   - code: crates/cclab-genesis/src/mcp/tools/run_change/mod.rs, crates/cclab-genesis/src/mcp/tools/fetch_issues.rs
4. **tag-union-validation** — Refactor spec validation to compositional tag-union logic
   - code: crates/cclab-genesis/src/services/spec_service.rs, crates/cclab-genesis/src/models/spec_rules.rs
5. **plus-diagrams-specs** — Add Class+, Sequence+, Flowchart+, Requirement+ diagrams to genesis specs
   - depends: tag-union-validation, openrpc-schema-sync
   - code: cclab/specs/cclab-genesis/create-proposal.md, cclab/specs/cclab-genesis/create-spec.md, cclab/specs/cclab-genesis/run-change/README.md, cclab/specs/cclab-genesis/implement-change.md

</proposal>
