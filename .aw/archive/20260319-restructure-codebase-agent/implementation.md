---
id: implementation
type: change_implementation
change_id: restructure-codebase-agent
---

# Implementation

## Summary

RestructureCodebaseAgent implementation across 9 files: (1) crates/cclab-agent/Cargo.toml (modified): added toml = "0.8" dependency for manifest parsing in ReadManifestTool; (2) crates/cclab-agent/src/agents/restructure_codebase.rs (new, 503 lines): RestructureCodebaseAgent — decomposes a codebase into budget-safe spec groups for SDD fillback flow; inner CodingAgent with 4 specialized tools (read_manifest, list_folder_summary, estimate_tokens, set_grouping); run() retry loop up to max_retries (default 2) on missing or empty grouping; builds retry prompt when set_grouping not called; default config: model=claude-sonnet-4-20250514, temperature=0.0, max_turns=40, max_retries=2, token_budget=50_000; builder pattern with_provider/with_provider_arc/with_model/with_temperature/with_max_turns/with_max_retries/with_token_budget/build(); system prompt SYSTEM_PROMPT enforces 5-step algorithm (read_manifest → list_folder_summary → estimate_tokens → drill down if over budget → set_grouping as terminal call); build_prompt() function constructs per-run user prompt with codebase path and budget; 7 unit tests with SetGroupingMockProvider (emits set_grouping tool call on first LLM call, stops on second) and NoGroupingMockProvider (always returns stop), covering success, retry, builder validation, and config defaults; (3) crates/cclab-agent/src/tools/estimate_tokens.rs (new, ~230 lines): EstimateTokensTool — heuristic token count via TOKENS_PER_LINE=3 multiplier; count_recursive() skips dot-prefixed hidden entries; count_lines() uses BufReader for efficiency; returns JSON { path, file_count, line_count, estimated_tokens, heuristic }; handles non-existent path with error field; 5 unit tests with tempfile including hidden file skip, directory recursion, single file, and execute() end-to-end; (4) crates/cclab-agent/src/tools/list_folder_summary.rs (new, ~334 lines): ListFolderSummaryTool — directory tree summary with file/line counts up to configurable depth (default 2); build_tree() recursively constructs TreeNode { name, path, is_dir, file_count, line_count, children }; at max_depth directories are counted via count_recursive() but children not expanded; sorted_entries for deterministic output; skips hidden entries; returns JSON { path, depth, total_file_count, total_line_count, tree }; handles file path (returns is_dir: false); 6 unit tests including depth-1 vs depth-2 expansion, file path input, non-existent path; (5) crates/cclab-agent/src/tools/read_manifest.rs (new, ~320 lines): ReadManifestTool — reads Cargo.toml (toml crate), package.json, pyproject.toml at given directory; parse_cargo_toml: extracts [workspace].members and [package].name, detects is_workspace; parse_package_json: handles workspaces as flat array or {"packages": [...]} object form; parse_pyproject_toml: extracts [project].name, surfaces hatch build targets as workspace_members; returns empty manifests array with guidance message when none found; 7 unit tests covering workspace/package Cargo.toml, both JSON workspace forms, pyproject, invalid inputs, execute with no manifests; (6) crates/cclab-agent/src/tools/set_grouping.rs (new, ~252 lines): SetGroupingTool + SpecGroup + GroupingState — terminal tool for finalizing grouping artifact; SpecGroup { name: String, paths: Vec<String>, description: String, estimated_tokens: Option<u64> } with Serialize/Deserialize/Clone/PartialEq; GroupingState { groups: Option<Vec<SpecGroup>> } in Arc<Mutex<...>>; SetGroupingTool::new(state: Arc<Mutex<GroupingState>>) bound to shared state; execute() deserializes SetGroupingArgs, maps to SpecGroup vec, stores in state, returns { status: "grouping_complete", group_count, message }; overwrites previous groups on repeat calls; 5 unit tests covering multi-group store, overwrite, empty groups, optional tokens, SpecGroup round-trip; (7) crates/cclab-agent/src/agents/mod.rs (modified): changed `mod restructure` to `pub mod restructure`, added `pub mod restructure_codebase`, added pub use restructure_codebase::{RestructureCodebaseAgent, RestructureCodebaseAgentBuilder, RestructureCodebaseAgentConfig}; (8) crates/cclab-agent/src/lib.rs (modified): added RestructureCodebaseAgent/Builder/Config to pub use agents::{...}, added pub use tools::{EstimateTokensTool, GroupingState, ListFolderSummaryTool, ReadManifestTool, SetGroupingTool, SpecGroup} with comment "Re-export codebase restructuring tools"; (9) crates/cclab-agent/src/tools/mod.rs (modified): added mod estimate_tokens, mod list_folder_summary, mod read_manifest, pub mod set_grouping declarations; added re-exports EstimateTokensTool, ListFolderSummaryTool, ReadManifestTool, GroupingState/SetGroupingTool/SpecGroup from set_grouping.

## Diff

```diff
diff --git a/crates/cclab-agent/Cargo.toml b/crates/cclab-agent/Cargo.toml
index d7f3b04f..32c4c63c 100644
--- a/crates/cclab-agent/Cargo.toml
+++ b/crates/cclab-agent/Cargo.toml
@@ -51,6 +51,9 @@ uuid = { version = "1", features = ["v4"] }
 # Token counting
 tiktoken-rs = "0.9"
 
+# TOML parsing (for manifest tool)
+toml = "0.8"
+
 # JSON schema validation (for structured output)
 jsonschema = "0.45"

diff --git a/crates/cclab-agent/src/agents/mod.rs b/crates/cclab-agent/src/agents/mod.rs
index e82b1bf8..b68b11b5 100644
--- a/crates/cclab-agent/src/agents/mod.rs
+++ b/crates/cclab-agent/src/agents/mod.rs
@@ -8,8 +8,9 @@ pub mod codebase_to_spec;
 pub mod crr;
 pub mod reference_codebase_context;
 pub mod reference_spec_context;
+pub mod restructure;
+pub mod restructure_codebase;
 pub mod review;
-mod restructure;
 
 // Keep the old module path accessible so existing code compiles without changes.
@@ -41,6 +42,9 @@ pub use restructure::{
     Clarification, Question, RestructureAgent, RestructureAgentBuilder, RestructureAgentConfig,
     RestructureInput, RestructureOutput, SpecExcerpt, SpecStore, StructuredIssue,
 };
+pub use restructure_codebase::{
+    RestructureCodebaseAgent, RestructureCodebaseAgentBuilder, RestructureCodebaseAgentConfig,
+};
 pub use review::{

diff --git a/crates/cclab-agent/src/lib.rs b/crates/cclab-agent/src/lib.rs
index 29e9c075..8e40f1dc 100644
--- a/crates/cclab-agent/src/lib.rs
+++ b/crates/cclab-agent/src/lib.rs
@@ -95,6 +95,7 @@ pub use agents::{
     ReferenceSpecContextAgent, ReferenceSpecContextAgentBuilder, ReferenceSpecContextAgentConfig,
     RestructureAgent, RestructureAgentBuilder, RestructureAgentConfig,
     RestructureInput, RestructureOutput, SpecExcerpt, SpecStore, StructuredIssue,
+    RestructureCodebaseAgent, RestructureCodebaseAgentBuilder, RestructureCodebaseAgentConfig,
     ReviewAgent, ReviewAgentBuilder, ReviewAgentConfig, ReviewIssue, ReviewType, ReviewVerdict,
     Reviewer, Severity,
 };
@@ -128,6 +129,9 @@ pub use tools::{
 // Re-export analysis tools
 pub use tools::{AskUserTool, RecordFindingTool, TakeNoteTool, WebFetchTool, WebSearchTool};
 
+// Re-export codebase restructuring tools
+pub use tools::{EstimateTokensTool, GroupingState, ListFolderSummaryTool, ReadManifestTool, SetGroupingTool, SpecGroup};
+
 // Re-export core types
 pub use types::{AgentId, Message, Role, ToolCall, ToolResult, TokenUsage};

diff --git a/crates/cclab-agent/src/tools/mod.rs b/crates/cclab-agent/src/tools/mod.rs
index 1d561479..0f8bd2fd 100644
--- a/crates/cclab-agent/src/tools/mod.rs
+++ b/crates/cclab-agent/src/tools/mod.rs
@@ -4,8 +4,12 @@
 
 mod analysis;
 mod bash;
+mod estimate_tokens;
 mod file;
+mod list_folder_summary;
+mod read_manifest;
 mod registry;
+pub mod set_grouping;
 mod tool;
 
 // Coding tools
@@ -15,6 +19,12 @@ pub use file::{EditFileTool, GlobTool, GrepTool, ReadFileTool, WriteFileTool};
 // Analysis tools
 pub use analysis::{AskUserTool, RecordFindingTool, TakeNoteTool, WebFetchTool, WebSearchTool};
 
+// Codebase restructuring tools
+pub use estimate_tokens::EstimateTokensTool;
+pub use list_folder_summary::ListFolderSummaryTool;
+pub use read_manifest::ReadManifestTool;
+pub use set_grouping::{GroupingState, SetGroupingTool, SpecGroup};
+
 // Core tool infrastructure
 pub use registry::ToolRegistry;
 pub use tool::{Tool, ToolDefinition, ToolExecutor, ToolParameter};

diff --git a/crates/cclab-agent/src/agents/restructure_codebase.rs b/crates/cclab-agent/src/agents/restructure_codebase.rs
new file mode 100644
index 00000000..NEW
--- /dev/null
+++ b/crates/cclab-agent/src/agents/restructure_codebase.rs
@@ -0,0 +1,503 @@
+//! RestructureCodebaseAgent — decomposes a codebase into budget-safe spec groups.
+// Config: model=claude-sonnet-4-20250514, temperature=0.0, max_turns=40, max_retries=2, token_budget=50_000
+// run(codebase_path): builds inner CodingAgent with 4 tools → loops until set_grouping called
+//   → on success: serde_json::to_string_pretty(groups)
+//   → on empty groups: retry with corrective prompt
+//   → on missing set_grouping: retry with corrective prompt
+//   → after max_retries: NovaError::Other
+// build_inner_agent(): ToolRegistry with ReadManifestTool + ListFolderSummaryTool +
+//   EstimateTokensTool + SetGroupingTool(state.clone())
+// build_prompt(path, budget): formats 4-step workflow instruction
+// SYSTEM_PROMPT: 5-step algorithm enforced, set_grouping as mandatory terminal call
+// Builder: with_provider/with_provider_arc/with_model/with_temperature/
+//          with_max_turns/with_max_retries/with_token_budget/build()
+// Tests: 7 tests — SetGroupingMockProvider (2-call sequence), NoGroupingMockProvider,
+//   test_run_returns_groups_when_set_grouping_called, test_run_fails_when_never_called,
+//   test_run_retries_on_missing_grouping, test_builder_missing_provider,
+//   test_config_defaults, test_builder_with_overrides

diff --git a/crates/cclab-agent/src/tools/estimate_tokens.rs b/crates/cclab-agent/src/tools/estimate_tokens.rs
new file mode 100644
index 00000000..NEW
--- /dev/null
+++ b/crates/cclab-agent/src/tools/estimate_tokens.rs
@@ -0,0 +1,~230 @@
+//! EstimateTokensTool — heuristic token count for a file or directory.
+// TOKENS_PER_LINE = 3; count_recursive() skips dot-prefixed entries; count_lines() BufReader
+// execute(): { path, file_count, line_count, estimated_tokens, heuristic } or { path, error }
+// Tests: 5 tests — count_lines, count_recursive file/directory, hidden_files_skipped,
+//   execute_returns_estimate, execute_nonexistent_path, execute_single_file

diff --git a/crates/cclab-agent/src/tools/list_folder_summary.rs b/crates/cclab-agent/src/tools/list_folder_summary.rs
new file mode 100644
index 00000000..NEW
--- /dev/null
+++ b/crates/cclab-agent/src/tools/list_folder_summary.rs
@@ -0,0 +1,~334 @@
+//! ListFolderSummaryTool — summarizes directory structure up to configurable depth.
+// depth param default 2; build_tree() → TreeNode { name, path, is_dir, file_count, line_count, children }
+// At max_depth: count_recursive() used, children not expanded; sorted_entries for determinism
+// execute(): { path, depth, total_file_count, total_line_count, tree } or file shortcut or error
+// Tests: 6 tests — count_lines, count_recursive, build_tree_depth_1/2, execute_returns_summary,
+//   execute_nonexistent, execute_file_path

diff --git a/crates/cclab-agent/src/tools/read_manifest.rs b/crates/cclab-agent/src/tools/read_manifest.rs
new file mode 100644
index 00000000..NEW
--- /dev/null
+++ b/crates/cclab-agent/src/tools/read_manifest.rs
@@ -0,0 +1,~320 @@
+//! ReadManifestTool — parses Cargo.toml / package.json / pyproject.toml.
+// parse_cargo_toml: toml::from_str → [workspace].members, [package].name, is_workspace
+// parse_package_json: array workspaces or {"packages":[...]} form
+// parse_pyproject_toml: [project].name + hatch build targets as workspace_members
+// Returns { path, manifests: [...] } or { path, manifests: [], message } when none found
+// Tests: 7 tests — cargo workspace/package, json workspace array/object,
+//   pyproject, invalid inputs, execute_no_manifests

diff --git a/crates/cclab-agent/src/tools/set_grouping.rs b/crates/cclab-agent/src/tools/set_grouping.rs
new file mode 100644
index 00000000..NEW
--- /dev/null
+++ b/crates/cclab-agent/src/tools/set_grouping.rs
@@ -0,0 +1,~252 @@
+//! SetGroupingTool — terminal tool: stores final SpecGroup list in shared state.
+// SpecGroup { name, paths: Vec<String>, description, estimated_tokens: Option<u64> }
+// GroupingState { groups: Option<Vec<SpecGroup>> } — Default impl; Arc<Mutex<...>> shared
+// SetGroupingTool::new(Arc<Mutex<GroupingState>>) — owned state ref
+// execute(): deserialize SetGroupingArgs → Vec<SpecGroup> → state.lock().groups = Some(groups)
+//   → { status: "grouping_complete", group_count, message }
+// Tests: 5 tests — stores_groups, overwrites_previous, empty_groups, optional_tokens, spec_group_round_trip
```

## Review: restructure-codebase-agent-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: restructure-codebase-agent

**Summary**: 229 tests pass, compiles clean.

