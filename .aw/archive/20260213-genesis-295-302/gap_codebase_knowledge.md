---
change_id: genesis-295-302
type: gap_codebase_knowledge
created_at: 2026-02-13T07:53:55.941287+00:00
updated_at: 2026-02-13T07:53:55.941287+00:00
---

# Gap Analysis: Codebase vs Knowledge

## Convention violations

### HIGH severity

- **Tag-union validation** — Knowledge doc (spec-to-code/spec-model.md) defines compositional tag system where requirements = union of all tags. Code (`spec_service.rs:validate_spec_type_requirements`) uses type-based match with hardcoded OR alternatives instead. (#298)
- **Verdict naming** — Knowledge doc (verdict-unification.md) standardizes to APPROVED/REVIEWED/REJECTED. Code (`helpers.rs:extract_verdict`) still parses legacy names like NEEDS_REVISION via checkbox format. (#296, #298)

### MEDIUM severity

- **Plus diagram convention** — Knowledge doc (spec-to-code/spec-model.md) defines 6 Plus diagram types (Sequence+, Flowchart+, Class+, ERD+, Requirement+, Block+) with semantic metadata. No existing spec uses any Plus variant — all diagrams are plain Mermaid. (#295, #299, #300, #301)
- **Spec-to-code model mapping** — Knowledge doc defines struct-level mapping rules (MCP input vs service input, Option<T> markers, defaults). No spec documents internal struct definitions. (#295)

## Pattern mismatches

### HIGH severity

- **Legacy v1 dispatch** — Knowledge doc (run-change/README.md) notes 8-location checklist for StatePhase updates. Code has v1 paths that bypass this checklist (e.g. `create_v1_proposal` writes different frontmatter, `get_schema_version` routes to v2/v3). Knowledge doesn't document removal strategy. (#297)

### LOW severity

- **MCP tool filtering** — Knowledge doc (40-mcp/dynamic-config.md) describes stage-based tool filtering to reduce token usage. Code does not implement this filtering — all tools available at all stages. (Not directly in scope but related to #302 simplification)
