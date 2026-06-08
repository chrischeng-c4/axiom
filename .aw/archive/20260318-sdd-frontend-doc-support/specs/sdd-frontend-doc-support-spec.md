---
id: sdd-frontend-doc-support-spec
main_spec_ref: cclab-sdd/logic/change-spec.md
merge_strategy: replace
create_complete: true
filled_sections: [overview, changes]
---

# SDD Frontend & Doc Support — Implementation Spec

## Overview
<!-- type: overview lang: markdown -->

Phase 1 implementation of the SDD spec system redesign. Migrates spec directory structure to group-scoped, adds section type system infrastructure, and implements the first two CLI generators (overview + changes) as the foundation for all 17 types.

Scope:
1. **Directory migration**: Move specs from `changes/{id}/specs/` to `changes/{id}/groups/{group}/specs/`
2. **Section type enum**: Add 17 section types to `spec_rules.rs`, deprecate old `SpecType`
3. **Section annotation**: Parse `<!-- type: xxx lang: yyy -->` in spec files
4. **Fill order**: Hardcoded section type priority for deterministic fill sequence
5. **CLI generator infra**: Shared infrastructure for `--type`, `--sdd-id`, `--sdd-refs` flags
6. **Wave 1 generators**: `overview` and `changes` type generators
7. **Prompt architecture**: Base template + type-specific inserts as embedded YAML data

Out of scope (future phases):
- Wave 2-4 CLI generators (mermaid, API, frontend types)
- spec_plan in reference_context
- Cross-group deduplication
- Section selection rule engine
- Wireframe YAML DSL schema
- CEM / DTCG format support

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  # Directory structure migration
  - path: crates/cclab-sdd/src/tools/create_change_spec.rs
    action: MODIFY
    desc: Read/write specs from groups/{group}/specs/ instead of specs/
  - path: crates/cclab-sdd/src/tools/create_change_merge.rs
    action: MODIFY
    desc: Find specs in groups/*/specs/, strip fill_sections/filled_sections
  - path: crates/cclab-sdd/src/tools/common_change_impl.rs
    action: MODIFY
    desc: Build spec execution order from groups/{group}/specs/
  - path: crates/cclab-sdd/src/tools/common_change_spec.rs
    action: MODIFY
    desc: Update SKELETON template, ALL_SECTIONS, spec path helpers
  - path: crates/cclab-sdd/src/workflow/helpers.rs
    action: MODIFY
    desc: find_specs_to_merge reads from groups/*/specs/

  # Section type system
  - path: crates/cclab-sdd/src/models/spec_rules.rs
    action: MODIFY
    desc: Add SectionType enum (17 types), deprecate SpecType, add fill_order()
  - path: crates/cclab-sdd/src/models/section.rs
    action: CREATE
    desc: Section annotation parser (regex), SectionMeta struct

  # CLI generator infrastructure
  - path: crates/cclab-sdd/src/tools/create_change_spec.rs
    action: MODIFY
    desc: Add --type flag routing to per-type generators
  - path: crates/cclab-sdd/src/generators/mod.rs
    action: CREATE
    desc: Generator trait, shared infra (annotation injection, sdd-id/refs)
  - path: crates/cclab-sdd/src/generators/overview.rs
    action: CREATE
    desc: Overview generator (markdown prose, no code fence)
  - path: crates/cclab-sdd/src/generators/changes.rs
    action: CREATE
    desc: Changes generator (YAML file list with path + action)

  # Prompt architecture
  - path: crates/cclab-sdd/src/prompts/section_prompts.yaml
    action: CREATE
    desc: Per-type flag descriptions + guidance (embedded data)
  - path: crates/cclab-sdd/src/tools/create_change_spec.rs
    action: MODIFY
    desc: Base prompt template with type-specific insert injection

  # Tests
  - path: crates/cclab-sdd/src/models/section_tests.rs
    action: CREATE
    desc: Section annotation parsing tests
  - path: crates/cclab-sdd/src/generators/tests.rs
    action: CREATE
    desc: Generator output tests for overview + changes types
```

# Reviews
