# Task: Implement Spec 'spec-prep-base-snapshot' for Change 'sdd-merge-3way'

## Instructions

1. Read spec **spec-prep-base-snapshot**: `cclab/changes/sdd-merge-3way/groups/merge-3way/specs/spec-prep-base-snapshot.md`
2. Implement **production code only** (no `#[test]` functions) according to spec requirements
3. When done, run `cclab sdd workflow create-change-implementation sdd-merge-3way` to advance

## Change Targets

### crates/cclab-sdd/src/tools/spec_plan.rs
- **function `prepare_modify_spec`**: Return a tuple (working_content, Option<base_content>) instead of String. Before mutating frontmatter, clone the raw source content as base_content. Return (modified_content, Some(raw_source)) for modify specs. When source file is missing (fallback to create), return (content, None).
- **function `prepare_specs_from_plan`**: After calling prepare_modify_spec(), if base_content is Some, write it to {spec_id}.base.md alongside the working spec. The .base.md write happens in the same loop iteration, immediately before or after writing the working spec.
- **DO NOT MODIFY**: prepare_create_spec, deduplicate_spec_plans, read_all_spec_plans, resolve_section_rules

### crates/cclab-sdd/src/tools/spec_plan.rs
- **function `test_prepare_modify_creates_base_snapshot`**: New test: setup a modify spec entry with existing source file, call prepare_specs_from_plan(), verify {spec_id}.base.md exists alongside {spec_id}.md with unmodified source content
- **function `test_prepare_create_no_base_snapshot`**: New test: setup a create spec entry, call prepare_specs_from_plan(), verify no .base.md file is created
- **function `test_prepare_modify_missing_source_no_base`**: New test: setup a modify spec entry where source file does not exist (fallback to create), verify no .base.md is written
- **function `test_prepare_skip_already_prepared_no_duplicate_base`**: New test: pre-create the spec file, call prepare_specs_from_plan(), verify it skips (no .base.md written for already-prepared specs)

## CLI Commands

```
# Read spec
Read file: cclab/changes/sdd-merge-3way/groups/merge-3way/specs/spec-prep-base-snapshot.md

# Advance implementation workflow
cclab sdd workflow create-change-implementation sdd-merge-3way
```