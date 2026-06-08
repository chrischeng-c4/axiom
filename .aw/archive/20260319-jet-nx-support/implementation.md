---
id: implementation
type: change_implementation
change_id: jet-nx-support
---

# Implementation

## Summary

Implemented full Nx monorepo support for Jet across 7 files (issues linked to jet-nx-support spec).

**pkg_manager/nx.rs** (239L, created):
- `NxConfig`: parses `nx.json` via serde with `flatten` for forward-compatibility across Nx versions.
- Graph types: `NxProjectGraph` / `NxGraphData` / `NxProject` / `NxProjectData` / `NxDependency` — match the JSON envelope emitted by `nx graph --json`.
- `NxProjectGraph::topological_sort()`: Kahn's BFS algorithm computing dependency-first build order. Within each zero-in-degree tier, projects are sorted alphabetically for deterministic output.
- `project_names()`: returns all project names sorted alphabetically.
- `project_root(name)`: resolves the `data.root` path for a project node.
- `NxWorkspaceManager::discover(root)`: checks for `nx.json`; returns `None` when absent, `Err` when malformed, `Some(Self)` when valid — satisfies R1 workspace detection requirement.
- `NxWorkspaceManager::get_project_graph()`: shells out `nx graph --json` via `std::process::Command` in the workspace root, propagates stderr on failure, parses JSON into `NxProjectGraph` — satisfies R2 project graph integration.

**pkg_manager/nx_test.rs** (249L, created):
- 12 unit tests covering: alphabetical sort with no deps, linear chain (a→b→c), diamond dependency pattern, 4 independent projects, sorted `project_names`, `project_root` lookup, JSON parse for minimal and deps-carrying payloads, `NxWorkspaceManager` discover absent/minimal/full `nx.json`, and error propagation for malformed JSON.

**tests/nx_support.rs** (213L, created):
- 13 integration tests exercising the public API surface: `WorkspaceMode::detect` returns `Nx` when `nx.json` is present, `Jet` when `package.json` workspaces set, `Single` when neither present; Nx takes priority over Jet; `jet-workspace.yaml` triggers Jet mode; malformed `nx.json` propagates error; `NxWorkspaceManager` stores correct root path and returns `None` without `nx.json`; topological sort respects declared dependencies; JSON roundtrip preserves ordering.

**pkg_manager/workspace.rs** (modified, +30L):
- Added `WorkspaceMode` enum with variants `Nx(NxWorkspaceManager)`, `Jet(WorkspaceManager)`, `Single`.
- `WorkspaceMode::detect(root)`: implements spec-defined priority order — `nx.json` → Jet/npm workspace → single project. All three cases covered with error propagation for malformed configs.

**pkg_manager/mod.rs** (modified, +1L):
- Added `pub mod nx;` to expose the new module.

**lib.rs** (modified, +1L):
- Added `nx` to the crate-level `pub use pkg_manager::{ ... }` re-export.

**cli.rs** (modified, +80L):
- `jet install`: added `--nx` flag to force Nx mode overriding auto-detection. `WorkspaceMode::detect` dispatch — in Nx mode, prints workspace root and skips per-project install (Nx manages monorepo-level dependency installation at the root).
- `jet build`: added `--nx` flag and `--project <name>` flag for targeting a specific Nx project. `WorkspaceMode::detect` dispatch routes to `run_nx_build()` when operating in Nx mode.
- `run_nx_build()` async function: fetches project graph via `get_project_graph()`, computes topological build order, optionally filters to the named project, iterates projects in dependency order — for each, resolves project root from graph data, detects entry point, invokes bundler with `minify=true` and `source_maps=true`, runs define replacement + DCE + minification pipeline, writes content-hashed output filename (`main.{hash8}.js`), prints per-project status. Summary line reports built vs skipped count and total elapsed time.

## Diff

```diff
diff --git a/crates/cclab-jet/src/pkg_manager/nx.rs b/crates/cclab-jet/src/pkg_manager/nx.rs
--- /dev/null
+++ b/crates/cclab-jet/src/pkg_manager/nx.rs
@@ nx.rs: +239L — NxConfig (nx.json parser, serde flatten for forward-compat); NxProjectGraph/NxGraphData/NxProject/NxProjectData/NxDependency data types for `nx graph --json` output; NxProjectGraph::topological_sort() via Kahn's BFS (alphabetical tie-breaking); project_names() sorted; project_root() lookup; NxWorkspaceManager::discover(root) detects nx.json and parses config; NxWorkspaceManager::get_project_graph() shells out `nx graph --json` and parses JSON output

diff --git a/crates/cclab-jet/src/pkg_manager/nx_test.rs b/crates/cclab-jet/src/pkg_manager/nx_test.rs
--- /dev/null
+++ b/crates/cclab-jet/src/pkg_manager/nx_test.rs
@@ nx_test.rs: +249L — unit tests: topological_sort with no deps (alphabetical), linear chain, diamond, independent projects; project_names sorted; project_root lookup; JSON parse minimal and with deps; NxWorkspaceManager discover absent/minimal/full nx.json; malformed nx.json returns error

diff --git a/crates/cclab-jet/tests/nx_support.rs b/crates/cclab-jet/tests/nx_support.rs
--- /dev/null
+++ b/crates/cclab-jet/tests/nx_support.rs
@@ nx_support.rs: +213L — integration tests: WorkspaceMode::detect returns Nx when nx.json present; Jet when package.json workspaces set (no nx.json); Single when no workspace config; Nx takes priority over Jet; Jet from jet-workspace.yaml; error on malformed nx.json; NxWorkspaceManager stores root path; returns None without nx.json; graph topological sort respects deps; includes all projects; project_names sorted; JSON roundtrip preserves order

diff --git a/crates/cclab-jet/src/pkg_manager/workspace.rs b/crates/cclab-jet/src/pkg_manager/workspace.rs
--- a/crates/cclab-jet/src/pkg_manager/workspace.rs
+++ b/crates/cclab-jet/src/pkg_manager/workspace.rs
@@ workspace.rs: +30L — added WorkspaceMode enum (Nx(NxWorkspaceManager), Jet(WorkspaceManager), Single); WorkspaceMode::detect(root) implements priority order: nx.json → Jet workspace → Single project

diff --git a/crates/cclab-jet/src/pkg_manager/mod.rs b/crates/cclab-jet/src/pkg_manager/mod.rs
--- a/crates/cclab-jet/src/pkg_manager/mod.rs
+++ b/crates/cclab-jet/src/pkg_manager/mod.rs
@@ mod.rs: +1L — added `pub mod nx;` to module registry

diff --git a/crates/cclab-jet/src/lib.rs b/crates/cclab-jet/src/lib.rs
--- a/crates/cclab-jet/src/lib.rs
+++ b/crates/cclab-jet/src/lib.rs
@@ lib.rs: +1L — added `nx` to `pub use pkg_manager::{ ... nx ... }` re-export list

diff --git a/crates/cclab-jet/src/cli.rs b/crates/cclab-jet/src/cli.rs
--- a/crates/cclab-jet/src/cli.rs
+++ b/crates/cclab-jet/src/cli.rs
@@ cli.rs: +80L — `jet install`: added `--nx` flag (force Nx mode); WorkspaceMode::detect dispatch; Nx mode prints workspace root and skips per-project install (Nx manages dependencies); `jet build`: added `--nx` flag and `--project <name>` flag; WorkspaceMode::detect dispatch; run_nx_build() async fn: fetches nx graph, computes topological order, optionally filters to single project, iterates projects calling bundler with minify+source_maps, writes content-hashed output files, prints build summary
```

## Review: jet-nx-support-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: jet-nx-support

**Summary**: The implementation correctly handles Nx workspace detection, project graph parsing, and topological sorting. CLI commands for build and install are updated to support Nx. Tests are included.

### Checklist

- [PASS] Does it implement all requirements from the spec?
- [PASS] Are the tests comprehensive?
- [PASS] Does it follow codebase conventions?

