---
number: 1140
title: "feat(sdd): spec format validation — check-alignment Phase 1"
state: open
labels: [type:enhancement, priority:p1, crate:sdd]
group: "check-alignment-phase1"
---

# #1140 — feat(sdd): spec format validation — check-alignment Phase 1

## Summary

Add structural/format validation to `cclab sdd check-alignment`. Phase 1 focuses on deterministic checks with zero false positives — no code parsing needed, pure spec-internal validation.

### Validation Rules

#### Format Compliance

| Rule | Check | Method |
|------|-------|--------|
| `missing_section_annotation` | Every `## Heading` has `<!-- type: X lang: Y -->` | Parse structured section format |
| `duplicate_section` | No duplicate `## Heading` within same file | Heading count |
| `format_priority_violation` | `type: config` section has JSON Schema, not prose; `type: logic` has mermaid, not prose | Check content type matches declared lang |

#### Logical Duplicate & Conflict Detection

| Rule | Check | Method |
|------|-------|--------|
| `duplicate_definition` | Same tool/entity defined in multiple JSON blocks within one file (e.g. 3 RPC blocks for `sdd_workflow_create_change_merge`) | Parse all JSON blocks, group by `name` field, flag duplicates |
| `definition_conflict_required` | Same tool's `required` array differs between duplicate blocks (e.g. one has `audit_log`, another doesn't) | Compare `required` arrays across duplicate blocks |
| `definition_conflict_field_name` | Same logical field has different names across blocks (e.g. `specs_merged` vs `merged_specs`) | Compare `properties` keys across duplicate blocks, flag near-matches (edit distance) |
| `definition_conflict_schema` | Same field has different type/enum/format across blocks | Compare field schemas across duplicate blocks |
| `rpc_field_consistency` | OpenRPC `x-*` extension fields are consistent across duplicate definitions | Compare extension values |

### Architecture

Single `spec_alignment::check()` library function, callable from:
- CLI: `cclab sdd check-alignment [path]`
- Artifact tools: internal call after write (write-time validation)
- Merge workflow: pre-commit check

No regex — spec format is structured. Parse sections by heading + annotation, parse JSON blocks with `serde_json`, compare structured data.

### Output

Same format as `validate-spec-structure` (text + JSON).

```
FAIL  workflow-tools.md
  duplicate_definition: sdd_workflow_create_change_merge defined 3 times (lines 305, 377, 498)
  definition_conflict_field_name: "specs_merged" (line 325) vs "merged_specs" (line 515)
  definition_conflict_required: block 1 requires [status, specs_merged, archive_path, audit_log, next_actions], block 3 requires [status, merged_specs, archive_path]
  missing_section_annotation: ## Commands (line 32) has no <!-- type: ... --> annotation

OK    change-merge.md
OK    agents.md
OK    platform.md
```

### Acceptance Criteria

- Running on the 4 specs from #1136 catches all format + logical violations found manually:
  - 4 duplicate Overview sections in workflow-tools.md
  - 3 conflicting RPC definitions with field name mismatch
  - Missing section annotations
  - Prose in structured sections
- Zero false positives on well-formed specs
- `cclab sdd check-alignment` returns exit code 0 on clean, non-zero on violations

Depends on: nothing
Blocks: #1141 (Phase 2), #1142 (Phase 3)
