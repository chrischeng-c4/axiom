---
change_id: genesis-295-302
type: gap_codebase_spec
created_at: 2026-02-13T07:44:07.397371+00:00
updated_at: 2026-02-13T07:44:07.397371+00:00
---

# Gap Analysis: Codebase vs Spec

## Code exists, spec incomplete

### HIGH severity

- **`validate_spec_type_requirements()` in spec_service.rs** — Code uses type-based match with hardcoded OR alternatives. Spec says tag-union. (#298)
- **`create_v1_proposal()` in proposal_service.rs** — Entire v1 legacy path with different frontmatter format. Spec only mentions "deprecated" in one line. (#297)
- **`extract_verdict()` in helpers.rs** — Dual-format parsing (YAML frontmatter + checkbox). No spec coverage at all. (#300)
- **`parse_task_blocks()` + `build_task_execution_order()` in helpers.rs** — Complex DAG algorithm. Only sequence diagram exists, no Flowchart Plus. (#300)
- **`topological_sort()` in helpers.rs/proposal_service.rs** — Kahn's algorithm. Mentioned as text only, no pseudocode/diagram. (#300)
- **`add_executor_info()` in run_change/mod.rs** — Maps action→artifact→config→executor chain. Mentioned in README but no diagram. (#300)
- **`analyze_specs()` in helpers.rs/spec.rs** — Missing/pending spec detection algorithm. Decision flowchart exists but not the algorithm detail. (#300)
- **Response fields `mcp_tool`, `has_proposal`, `spec_count`, `missing_specs_count`, `has_specs_dir`, `review_file`** — Returned by code, not in OpenRPC schema. (#296)
- **`get_schema_version()` in run_change** — Routes between v2/v3 schema behavior. Undocumented. (#297)
- **`parse_affected_specs()` in run_change** — Parses v1 frontmatter format. Undocumented. (#297)

### MEDIUM severity

- **Internal structs `SpecPlanInput`, `ScopeAreaInput`, `CreateProposalInput`, `ImpactData`** — No Class Plus diagrams. (#295)
- **Internal structs `DiagramData`, `SpecFormatRules`, `DocumentType`** — No Class Plus diagrams. (#295)
- **Internal structs `TaskInfo`, `ClarifyRoute`, per-module Action enums** — No Class Plus diagrams. (#295)
- **Service↔MCP↔STATE.yaml layer interactions** — No Sequence Plus showing which layer writes what. (#299)
- **`workflow_version: 2` field** — Inconsistently set across helpers vs mod.rs. (#296)

## Spec exists, code missing

### HIGH severity

- **`issues` parameter on `run_change`** — Spec (fetch-issues.md §Description Resolution) defines regex parsing. Proposed: explicit `issues` param instead. (#302)

## Spec exists, no diagrams

### MEDIUM severity

- **Requirement Plus diagrams** — No spec has requirement traceability diagrams mapping R→scenario→code module. (#301)
- **Auto-tag table for EventDriven/Utility** — Spec says `[events, async]`, code does `[api, events, async]`. (#298)
