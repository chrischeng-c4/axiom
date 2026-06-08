---
id: implementation
type: change_implementation
change_id: jet-jit-runner
---

# Implementation

## Summary

Implement JIT execution + task runner for jet (#764). Added 9 new files across 2 modules (runner/, task_runner/) and modified 4 existing files. Runner module: ScriptRunner resolves package.json scripts with lifecycle hooks, JitEngine transforms TS/TSX/JSX via Tree-sitter to temp .js and executes via Node.js, watch mode via notify crate, inline source maps via base64. Task runner module: JetConfig parses jet.config.yaml pipeline, TaskGraph builds DAG with topological sort and cycle detection, TaskCache provides content-hash (SHA-256) based caching with output file restoration. CLI: added `jet run`, `jet exec`, `jet dlx` subcommands with --watch, --filter, --dry flags. 110 tests pass, 0 warnings.

## Diff

```diff
Modified: Cargo.lock (+1 base64 dep), Cargo.toml (+base64, tempfile moved to deps), lib.rs (+runner, +task_runner modules), cli.rs (+177 lines: run/exec/dlx commands + handle_run resolver).

New files:
- runner/mod.rs (199L): ScriptRunner with run_script, run_file, exec_command, lifecycle hooks
- runner/env.rs (56L): PATH injection (.bin), NODE_ENV, JET_* vars
- runner/jit.rs (148L): JitEngine - Tree-sitter transform, temp .js, Node.js execution, watch mode
- runner/source_map.rs (54L): Inline source map generation, identity mapping
- runner/watcher.rs (96L): DebouncedWatcher for file change detection
- task_runner/mod.rs (193L): TaskRunner orchestrator with parallel execution + caching
- task_runner/config.rs (108L): JetConfig/TaskDef YAML parser
- task_runner/graph.rs (248L): TaskGraph DAG, topological sort, cycle detection
- task_runner/cache.rs (149L): TaskCache - hash lookup, store, output restoration
- task_runner/hash.rs (102L): SHA-256 content hash computation
```

## Review: jet-jit-runner-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: jet-jit-runner

**Summary**: Implementation covers all 4 milestones from the spec. Script runner resolves package.json scripts with pre/post lifecycle hooks and .bin PATH injection. JIT engine transforms TS/TSX/JSX via existing Tree-sitter infrastructure, writes temp .js, executes via Node.js child process with inline source maps. Task runner parses jet.config.yaml pipeline, builds DAG with cycle detection and topological sort, executes with content-hash caching. CLI wires run/exec/dlx commands with --watch, --filter, --dry flags. 110 tests pass, 0 warnings.

### Checklist

- [PASS] jet run <script> executes package.json scripts
  - ScriptRunner.run_script() with lifecycle hooks
- [PASS] jet run file.ts executes TypeScript without compilation step
  - JitEngine transforms via Tree-sitter, executes temp .js via Node.js
- [PASS] Task graph respects dependsOn ordering
  - TaskGraph uses Kahn's algorithm for topological sort with cycle detection
- [PASS] Cached tasks skip execution and replay outputs
  - TaskCache with SHA-256 content hashing, store/lookup/restore
- [PASS] --watch mode re-runs on file changes
  - notify crate watcher in JitEngine.execute_watch()
- [PASS] jet exec runs arbitrary command with .bin on PATH
  - ScriptRunner.exec_command() resolves .bin path
- [PASS] jet dlx downloads and executes package
  - Creates temp project, installs, runs bin
- [PASS] 0 compiler warnings
- [PASS] All 110 tests pass

