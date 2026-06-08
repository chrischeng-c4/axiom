---
change_id: sdd-merge
type: gap_spec_knowledge
created_at: 2026-02-15T04:03:38.561524+00:00
updated_at: 2026-02-15T04:03:38.561524+00:00
---

# Gap Analysis: Spec vs Knowledge

## Spec Responsibilities vs Knowledge Architecture

### GAP-SK-1: No spec for sdd_generate_* tool renaming impact (severity: high)
- **Spec side**: spec_context identifies aurora_generate_* → sdd_generate_* rename but no spec defines the migration strategy
- **Knowledge side**: 40-mcp/dynamic-config.md defines stage-based tool filtering by tool name prefix — renaming breaks existing filter rules
- **Impact**: All MCP tool filter configs referencing `aurora_generate_*` must be updated

### GAP-SK-2: Knowledge documents SpecIR as Aurora-owned, specs define it in Genesis (severity: high)
- **Spec side**: spec-ir-yaml-schema (cclab-genesis) and prism-yaml-codegen (cclab-prism) define YAML SpecIR as genesis-owned
- **Knowledge side**: spec-to-code/code-generator-contract.md and genesis-372-impact.md still reference Aurora as SpecIR owner
- **Impact**: Knowledge docs need updating after merge to reflect cclab-sdd ownership

### GAP-SK-3: Legacy generator coupling not addressed in specs (severity: medium)
- **Spec side**: prism-codegen-unification spec addresses migrating generators to Prism but doesn't cover legacy FastAPI/Express/Axum generators
- **Knowledge side**: genesis-325-329/gap_codebase_knowledge.md identifies generators still coupled to JSON Schema instead of SpecIR
- **Impact**: Legacy generators in Aurora that get moved to sdd will still bypass the SpecIR pipeline

### GAP-SK-4: Merge workflow doesn't collect YAML manifests (severity: medium)
- **Spec side**: merge-change spec (cclab-genesis) only describes .md file collection for merging
- **Knowledge side**: Knowledge pitfall notes merge logic ignores spec_ir/*.yaml files
- **Impact**: YAML IR manifests won't be archived/merged properly after crate rename

## Summary

4 gaps identified: 2 high severity (tool rename impact, SpecIR ownership docs), 2 medium severity (legacy generator coupling, YAML merge gap). No design proposals included — gaps are factual observations only.