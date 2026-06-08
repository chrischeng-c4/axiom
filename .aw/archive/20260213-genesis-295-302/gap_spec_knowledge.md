---
change_id: genesis-295-302
type: gap_spec_knowledge
created_at: 2026-02-13T07:54:28.484880+00:00
updated_at: 2026-02-13T07:54:28.484880+00:00
---

# Gap Analysis: Spec vs Knowledge

## Knowledge patterns not reflected in specs

### HIGH severity

- **Plus diagram types** — Knowledge (spec-to-code/spec-model.md) defines 6 Plus diagram types with semantic metadata schemas (SemanticType nodes, layer annotations, traceability links). No existing genesis spec uses any Plus variant. All diagrams are plain Mermaid. (#295, #299, #300, #301)
- **Compositional tag system** — Knowledge defines tag-union validation where requirements = union(auto-tags + explicit tags). Spec (create-spec.md) documents this rule but the spec's own acceptance criteria don't test tag-union behavior — they test per-type validation. (#298)

### MEDIUM severity

- **Spec-to-code struct mapping** — Knowledge (spec-to-code/code-generator-contract.md) defines how spec requirements map to Rust structs (Option<T>, Vec<T>, defaults). No genesis spec documents its internal struct definitions in a machine-readable way. (#295)
- **MCP tool filtering by stage** — Knowledge (40-mcp/dynamic-config.md) defines stage-based tool sets. No spec references this filtering. run-change spec exposes all tools regardless of stage.

## Spec responsibilities contradicting knowledge

### HIGH severity

- **fetch-issues as standalone action** — Spec (fetch-issues.md) defines `genesis_fetch_issues` as a separate MCP tool called via `action: "fetch_issues"`. Knowledge (run-change/README.md) describes it as part of the run_change flow. Proposed: merge into run_change with explicit `issues` param. (#302)

### MEDIUM severity

- **Legacy v1 spec coverage** — Spec (create-proposal.md) mentions v1 as "deprecated" in one line. Knowledge (run-change/README.md) documents v1 as actively maintained with 8-location checklist. Mismatch: spec says deprecated, knowledge says still active. (#297)
