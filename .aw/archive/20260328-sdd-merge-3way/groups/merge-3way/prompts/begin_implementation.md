# Task: Begin Implementation for Change 'sdd-merge-3way'

## Instructions

1. List all change specs in `cclab/changes/sdd-merge-3way/`
2. Read spec **merge-3way** to understand requirements: `cclab/changes/sdd-merge-3way/groups/merge-3way/specs/merge-3way.md`
3. Implement **production code only** (no `#[test]` functions) for each change spec in order, starting with **merge-3way**
4. When done with merge-3way, run `cclab sdd workflow create-change-implementation sdd-merge-3way` to advance

## Change Targets

### crates/cclab-sdd/src/tools/create_change_merge.rs
- **function `execute_workflow`**: After Strip step, check for .base.md sibling file. If present and git available, invoke git merge-file with temp files (ours=current main spec, base=.base.md, theirs=cleaned change spec). Buffer merge results. On conflict, record spec_id. After loop, abort if any conflicts. Otherwise proceed to write pass.
- **function `merge_3way`**: New function: takes ours/base/theirs content strings, writes to tempdir, invokes git merge-file, returns Result<String> for clean merge or Err with conflict details
- **function `find_git_binary`**: New function: locate git binary on PATH, return Option<PathBuf>
- **DO NOT MODIFY**: workflow_definition, build_archive_path

### crates/cclab-sdd/src/workflow/helpers.rs
- **function `collect_spec_paths_into`**: Add filter: skip files ending with .base.md extension pattern
- **DO NOT MODIFY**: find_specs_to_merge, collect_spec_files, next_action, format_cli_command

### crates/cclab-sdd/src/tools/create_change_merge.rs
- **function `test_3way_merge_clean`**: New test: setup base + diverged main + change spec, verify clean merge produces expected content
- **function `test_3way_merge_conflict`**: New test: setup conflicting content, verify merge aborts with conflict report
- **function `test_base_md_skipped_by_find_specs`**: New test: verify .base.md files are not included in find_specs_to_merge() results
- **function `test_no_base_fallback_overwrite`**: New test: verify specs without .base.md use overwrite behavior (backward compat)
- **function `test_audit_log_3way_merge`**: New test: verify audit log records '3way-merge' action for successful 3-way merges

## CLI Commands

```
# Read spec
Read file: cclab/changes/sdd-merge-3way/groups/merge-3way/specs/merge-3way.md

# Advance implementation workflow
cclab sdd workflow create-change-implementation sdd-merge-3way
```