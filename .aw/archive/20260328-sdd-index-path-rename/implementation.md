---
id: implementation
type: change_implementation
change_id: sdd-index-path-rename
---

# Implementation

## Summary

Renamed Lens index storage path from `.cclab/lens/` to `cclab/.index/` and added code intelligence CLI hints to implementation prompts.

**Index Path Rename** (`.cclab/lens/` → `cclab/.index/`):

1. **`storage.rs`** — Core path resolution: `resolve_lens_storage` now joins `cclab/.index` instead of `.cclab/lens`. All doc comments and 5 test assertions updated.

2. **`search/mod.rs`** — `resolve_dir` path construction and doc comments updated to `cclab/.index/search_index/`.

3. **`lint/custom.rs`** — `load_from_workspace` path construction and module-level doc comments updated to `cclab/.index/rules.toml`.

4. **`server/daemon.rs`** — `default_socket_path` doc comment updated.

5. **`cclab-sdd-cli/daemon.rs`** — Error context string updated.

6. **`.gitignore`** — Entry renamed from `.cclab/lens/` to `cclab/.index/`.

7. **Specs updated** — `lens-index-storage.md` overview, requirements, acceptance criteria, and Mermaid diagram all reference `{project_root}/cclab/.index/`.

**CLI Hints in Implementation Prompt**:

8. **`create_change_impl.rs`** — `build_implement_code_prompt` resolves executor before building prompt. When executor is `["mainthread"]`, appends 5 code intelligence CLI commands (`symbols`, `hover`, `references`, `impact`, `context`) inside the CLI Commands code block. Non-mainthread executors get no hints.

9. **`implement-task.md` spec** — Added `{{#if executor == mainthread}}` conditional Code Intelligence section.

10. **6 new tests** — CLI hints presence for mainthread, all 5 commands present, hints inside CLI Commands block, subsequent spec also gets hints, tests prompt has no hints, write-diff prompt has no hints.

## Diff

```diff
diff --git a/.gitignore b/.gitignore
--- a/.gitignore
+++ b/.gitignore
@@ .gitignore: Rename lens daemon index entry
-.cclab/lens/
+cclab/.index/

diff --git a/cclab/specs/crates/cclab-sdd/logic/implement-task.md b/cclab/specs/crates/cclab-sdd/logic/implement-task.md
--- a/cclab/specs/crates/cclab-sdd/logic/implement-task.md
+++ b/cclab/specs/crates/cclab-sdd/logic/implement-task.md
@@ implement-task.md: +Code Intelligence CLI hints block gated by {{#if executor == mainthread}}
@@ implement-task.md: Lists cclab sdd symbols/hover/references/impact/context commands

diff --git a/cclab/specs/crates/cclab-sdd/logic/lens-index-storage.md b/cclab/specs/crates/cclab-sdd/logic/lens-index-storage.md
--- a/cclab/specs/crates/cclab-sdd/logic/lens-index-storage.md
+++ b/cclab/specs/crates/cclab-sdd/logic/lens-index-storage.md
@@ lens-index-storage.md: Overview — ~/.cclab/projects/{path_hash}/lens/ → {project_root}/cclab/.index/
@@ lens-index-storage.md: R1 requirement path updated
@@ lens-index-storage.md: Acceptance Criteria scenarios updated
@@ lens-index-storage.md: Mermaid diagram node text updated

diff --git a/crates/cclab-sdd-cli/src/daemon.rs b/crates/cclab-sdd-cli/src/daemon.rs
--- a/crates/cclab-sdd-cli/src/daemon.rs
+++ b/crates/cclab-sdd-cli/src/daemon.rs
@@ daemon.rs (cli): Error context string .cclab/lens/ → cclab/.index/

diff --git a/crates/cclab-sdd/src/lint/custom.rs b/crates/cclab-sdd/src/lint/custom.rs
--- a/crates/cclab-sdd/src/lint/custom.rs
+++ b/crates/cclab-sdd/src/lint/custom.rs
@@ custom.rs: Module doc comment .cclab/lens/rules.toml → cclab/.index/rules.toml
@@ custom.rs: Example doc block path updated
@@ custom.rs: load_from_workspace convenience loader doc updated
@@ custom.rs: Path construction .join(".cclab").join("lens") → .join("cclab").join(".index")

diff --git a/crates/cclab-sdd/src/search/mod.rs b/crates/cclab-sdd/src/search/mod.rs
--- a/crates/cclab-sdd/src/search/mod.rs
+++ b/crates/cclab-sdd/src/search/mod.rs
@@ search/mod.rs: SEARCH_INDEX_DIR doc comment .cclab/lens/ → cclab/.index/
@@ search/mod.rs: save_index doc comment updated
@@ search/mod.rs: resolve_dir path construction .join(".cclab").join("lens") → .join("cclab").join(".index")

diff --git a/crates/cclab-sdd/src/server/daemon.rs b/crates/cclab-sdd/src/server/daemon.rs
--- a/crates/cclab-sdd/src/server/daemon.rs
+++ b/crates/cclab-sdd/src/server/daemon.rs
@@ server/daemon.rs: default_socket_path doc comment .cclab/lens/daemon.sock → cclab/.index/daemon.sock

diff --git a/crates/cclab-sdd/src/storage.rs b/crates/cclab-sdd/src/storage.rs
--- a/crates/cclab-sdd/src/storage.rs
+++ b/crates/cclab-sdd/src/storage.rs
@@ storage.rs: Module doc comment .cclab/lens/ → cclab/.index/
@@ storage.rs: resolve_lens_storage — .join(".cclab").join("lens") → .join("cclab").join(".index")
@@ storage.rs: resolve_module_index doc comment updated
@@ storage.rs: resolve_pid_file doc comment updated
@@ storage.rs: resolve_socket_path doc comment updated
@@ storage.rs: resolve_cache_dir doc comment updated
@@ storage.rs: All 5 test assertions updated from .cclab/lens → cclab/.index

diff --git a/crates/cclab-sdd/src/tools/create_change_impl.rs b/crates/cclab-sdd/src/tools/create_change_impl.rs
--- a/crates/cclab-sdd/src/tools/create_change_impl.rs
+++ b/crates/cclab-sdd/src/tools/create_change_impl.rs
@@ create_change_impl.rs: build_implement_code_prompt — resolve executor before prompt, gate CLI hints on mainthread
@@ create_change_impl.rs: +cli_hints static string with 5 code intelligence commands (symbols/hover/references/impact/context)
@@ create_change_impl.rs: format! macro appends {cli_hints} inside CLI Commands code block
@@ create_change_impl.rs: +test test_impl_prompt_contains_cli_hints_for_mainthread
@@ create_change_impl.rs: +test test_impl_prompt_cli_hints_contains_all_commands
@@ create_change_impl.rs: +test test_impl_prompt_cli_hints_inside_cli_commands_block
@@ create_change_impl.rs: +test test_impl_prompt_subsequent_spec_also_has_hints
@@ create_change_impl.rs: +test test_tests_prompt_no_cli_hints
@@ create_change_impl.rs: +test test_write_diff_prompt_no_cli_hints
```
