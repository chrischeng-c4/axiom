---
id: jet-jit-runner-spec
main_spec_ref: cclab-jet/jit-runner.md
merge_strategy: new
fill_sections: [overview, diagrams, api_spec, changes]
filled_sections: [overview, diagrams, api_spec, changes]
create_complete: true
---

# Jet Jit Runner Spec

## Overview


Add JIT execution (run JS/TS without pre-build) and task runner (parallel script orchestration with caching) to jet, comparable to Bun run + Turborepo.

### Schemas

#### JetConfig (jet.config.yaml)

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "jet://schemas/jet-config",
  "type": "object",
  "properties": {
    "pipeline": {
      "type": "object",
      "description": "Task definitions: task_name → TaskDef",
      "additionalProperties": { "$ref": "#/$defs/TaskDef" }
    }
  },
  "$defs": {
    "TaskDef": {
      "type": "object",
      "properties": {
        "dependsOn": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Task dependencies. ^task means cross-package dep."
        },
        "inputs": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Glob patterns for input files (e.g. ['src/**', 'package.json'])"
        },
        "outputs": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Glob patterns for output files (e.g. ['dist/**'])"
        },
        "cache": { "type": "boolean", "default": true },
        "persistent": {
          "type": "boolean",
          "default": false,
          "description": "Long-running task (dev servers), never cached"
        },
        "env": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Env vars that affect cache key"
        }
      }
    }
  }
}
```

#### TaskResult

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "jet://schemas/task-result",
  "type": "object",
  "required": ["task_name", "status"],
  "properties": {
    "task_name": { "type": "string" },
    "package_name": { "type": ["string", "null"] },
    "status": { "enum": ["success", "failed", "cached", "skipped"] },
    "duration_ms": { "type": "integer" },
    "cache_hit": { "type": "boolean" },
    "exit_code": { "type": "integer" },
    "stdout": { "type": "string" },
    "stderr": { "type": "string" }
  }
}
```

#### TaskCacheEntry

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "jet://schemas/task-cache-entry",
  "type": "object",
  "required": ["hash", "task_name", "outputs"],
  "properties": {
    "hash": { "type": "string", "description": "SHA-256 of inputs + env + command + deps hashes" },
    "task_name": { "type": "string" },
    "package_name": { "type": ["string", "null"] },
    "outputs": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Relative paths of cached output files"
    },
    "stdout": { "type": "string" },
    "stderr": { "type": "string" },
    "created_at": { "type": "string", "format": "date-time" }
  }
}
```

#### JitOptions

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "jet://schemas/jit-options",
  "type": "object",
  "properties": {
    "entry": { "type": "string", "description": "File path to execute (e.g. file.ts)" },
    "watch": { "type": "boolean", "default": false },
    "args": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Arguments passed to the executed script"
    },
    "env": {
      "type": "object",
      "additionalProperties": { "type": "string" },
      "description": "Additional env vars"
    }
  }
}
```

#### ScriptRunOptions

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "jet://schemas/script-run-options",
  "type": "object",
  "required": ["script_name"],
  "properties": {
    "script_name": { "type": "string" },
    "args": { "type": "array", "items": { "type": "string" } },
    "filter": { "type": ["string", "null"], "description": "Workspace filter pattern" },
    "dry_run": { "type": "boolean", "default": false }
  }
}
```
## Diagrams

### Sequence Diagram

#### jet run \<script\> — Script Resolution

```mermaid
sequenceDiagram
    participant U as User
    participant CLI as jet CLI
    participant PM as PackageManager
    participant FS as FileSystem
    participant Node as Node.js

    U->>CLI: jet run <name> [args]
    CLI->>PM: lookup package.json scripts
    alt Script found in package.json
        PM->>CLI: script command string
        CLI->>CLI: inject pre<name> hook
        CLI->>Node: sh -c "<command>" (with .bin on PATH)
        Node-->>CLI: exit code
        CLI->>CLI: inject post<name> hook
    else File exists on disk
        CLI->>FS: check file extension
        alt .ts / .tsx / .jsx
            FS-->>CLI: TypeScript/JSX file
            CLI->>CLI: JIT transform (Tree-sitter)
            CLI->>Node: execute transformed .js
        else .js / .mjs
            FS-->>CLI: JavaScript file
            CLI->>Node: execute directly
        end
        Node-->>CLI: exit code
    else Not found
        CLI-->>U: error: script not found
    end
    CLI-->>U: exit code
```

#### jet run \<task\> — Task Runner with Caching

```mermaid
sequenceDiagram
    participant U as User
    participant CLI as jet CLI
    participant TR as TaskRunner
    participant TG as TaskGraph
    participant Cache as TaskCache
    participant Exec as Executor

    U->>CLI: jet run build --filter=pkg-a
    CLI->>TR: run("build", filter="pkg-a")
    TR->>TG: load jet.config.yaml pipeline
    TG->>TG: build DAG + topological sort
    TG-->>TR: ordered task list

    loop For each task (parallel where possible)
        TR->>Cache: compute_hash(inputs, env, cmd, deps)
        Cache-->>TR: hash
        TR->>Cache: lookup(hash)
        alt Cache hit
            Cache-->>TR: cached outputs + stdout/stderr
            TR->>TR: restore outputs to disk
            TR->>U: replay stdout (CACHED)
        else Cache miss
            TR->>Exec: spawn(command, env)
            Exec-->>TR: exit_code, stdout, stderr
            TR->>Cache: store(hash, outputs, stdout, stderr)
            TR->>U: stream stdout/stderr
        end
    end
    TR-->>U: summary (success/failed/cached counts)
```

### Flowchart

#### JIT Transform Pipeline

```mermaid
flowchart TD
    A[jet run file.ts] --> B{File extension?}
    B -->|.ts/.tsx| C[Read source file]
    B -->|.jsx| C
    B -->|.js/.mjs| G[Execute directly via Node.js]

    C --> D[Tree-sitter parse AST]
    D --> E[Strip type annotations]
    E --> E2{Contains JSX?}
    E2 -->|Yes| E3[Transform JSX to createElement]
    E2 -->|No| F
    E3 --> F[Generate inline source map]
    F --> F2[Write temp .js file]
    F2 --> G2[Execute temp .js via Node.js]
    G2 --> H{Watch mode?}
    G --> H
    H -->|Yes| I[Watch file changes]
    I --> |File changed| C
    H -->|No| J[Return exit code + cleanup]
```

### Class Diagram

```mermaid
classDiagram
    class ScriptRunner {
        -project_root: PathBuf
        -pkg_json: PackageJson
        +run_script(name, args) Result~ExitCode~
        +run_file(path, args) Result~ExitCode~
        +exec_command(cmd, args) Result~ExitCode~
        -resolve_bin_path(cmd) Option~PathBuf~
        -inject_env() HashMap~String,String~
    }

    class JitEngine {
        -transformer: TreeSitterTransformer
        -temp_dir: TempDir
        +execute(entry, opts: JitOptions) Result~ExitCode~
        +execute_watch(entry, opts) Result~()~
        -transform_file(path) Result~PathBuf~
    }

    class TreeSitterTransformer {
        -ts_parser: Parser
        -tsx_parser: Parser
        +transform(source, lang) Result~TransformOutput~
        -strip_types(tree, source) String
        -transform_jsx(tree, source) String
        -generate_source_map(original, transformed) String
    }

    class TransformOutput {
        +code: String
        +source_map: String
        +temp_path: PathBuf
    }

    class TaskRunner {
        -config: JetConfig
        -graph: TaskGraph
        -cache: TaskCache
        +run(task_name, filter, dry_run) Result~Vec~TaskResult~~
        -execute_task(task) Result~TaskResult~
    }

    class TaskGraph {
        -tasks: HashMap~String,TaskDef~
        -edges: Vec~(String,String)~
        +from_config(pipeline) Result~Self~
        +topological_sort() Result~Vec~String~~
        +get_ready_tasks(completed) Vec~String~
        -detect_cycles() Result~()~
    }

    class TaskCache {
        -cache_dir: PathBuf
        +compute_hash(task, inputs, env) String
        +lookup(hash) Option~TaskCacheEntry~
        +store(hash, entry) Result~()~
        +restore_outputs(entry, target) Result~()~
    }

    ScriptRunner --> JitEngine : delegates .ts/.tsx
    JitEngine --> TreeSitterTransformer : uses
    TreeSitterTransformer --> TransformOutput : produces
    TaskRunner --> TaskGraph : builds
    TaskRunner --> TaskCache : checks/stores
    ScriptRunner --> TaskRunner : delegates pipeline tasks
```

### State Diagram

#### Task Execution States

```mermaid
stateDiagram-v2
    [*] --> Pending : task enqueued
    Pending --> Waiting : has unmet dependencies
    Pending --> HashComputing : all deps satisfied

    Waiting --> HashComputing : deps completed

    HashComputing --> CacheChecking : hash computed
    CacheChecking --> Cached : cache hit
    CacheChecking --> Running : cache miss

    Running --> Success : exit code 0
    Running --> Failed : exit code != 0

    Cached --> [*] : restore outputs + replay
    Success --> [*] : store in cache
    Failed --> [*] : report error
    Skipped --> [*] : dep failed

    Waiting --> Skipped : dep failed
    Pending --> Skipped : --filter excluded
```
## API Spec

### OpenAPI 3.1

N/A — jet is a CLI tool, not a web service.

### OpenRPC 1.3

N/A

### AsyncAPI 2.6

N/A

### Serverless Workflow 0.8

Task runner orchestration modeled as a serverless workflow:

```yaml
id: jet-task-runner
version: '0.8'
specVersion: '0.8'
name: Jet Task Runner Pipeline
description: Parallel task execution with dependency resolution and caching
start: resolve-tasks

functions:
  - name: loadConfig
    operation: jet://internal/load-jet-config
  - name: buildGraph
    operation: jet://internal/build-task-graph
  - name: computeHash
    operation: jet://internal/compute-content-hash
  - name: checkCache
    operation: jet://internal/check-task-cache
  - name: executeTask
    operation: jet://internal/execute-task
  - name: storeCache
    operation: jet://internal/store-task-cache
  - name: restoreOutputs
    operation: jet://internal/restore-cached-outputs

states:
  - name: resolve-tasks
    type: operation
    actions:
      - functionRef: loadConfig
      - functionRef: buildGraph
    transition: schedule-tasks

  - name: schedule-tasks
    type: switch
    dataConditions:
      - condition: "${ .ready_tasks | length > 0 }"
        transition: execute-batch
      - condition: "${ .all_complete == true }"
        transition: report-results
    defaultCondition:
      transition: wait-for-deps

  - name: wait-for-deps
    type: sleep
    duration: PT0.1S
    transition: schedule-tasks

  - name: execute-batch
    type: parallel
    branches:
      - name: per-task
        actions:
          - functionRef: computeHash
          - subFlowRef: cache-or-run
    completionType: allOf
    transition: schedule-tasks

  - name: cache-or-run
    type: switch
    dataConditions:
      - condition: "${ .cache_hit == true }"
        transition: restore-from-cache
    defaultCondition:
      transition: run-task

  - name: restore-from-cache
    type: operation
    actions:
      - functionRef: restoreOutputs
    end: true

  - name: run-task
    type: operation
    actions:
      - functionRef: executeTask
      - functionRef: storeCache
    end: true

  - name: report-results
    type: operation
    end: true
```
## Changes

### Milestone 1: Script Runner

| File | Action | Description |
|------|--------|-------------|
| `crates/cclab-jet/src/runner/mod.rs` | Create | ScriptRunner: resolve package.json scripts, inject .bin PATH, lifecycle hooks (pre/post) |
| `crates/cclab-jet/src/runner/env.rs` | Create | Environment injection: NODE_ENV, JET_* variables, .bin PATH construction |
| `crates/cclab-jet/src/cli.rs` | Modify | Add `run <script> [args]`, `exec <cmd>`, `dlx <pkg>` subcommands |
| `crates/cclab-jet/src/lib.rs` | Modify | Export runner module |

### Milestone 2: JIT TypeScript/JSX Execution

| File | Action | Description |
|------|--------|-------------|
| `crates/cclab-jet/src/runner/jit.rs` | Create | JitEngine: transform TS/TSX via Tree-sitter, write temp .js, execute via Node.js child process |
| `crates/cclab-jet/src/runner/source_map.rs` | Create | Inline source map generation for transformed files |
| `crates/cclab-jet/src/runner/watcher.rs` | Create | Watch mode: file change detection → re-transform → restart Node.js process |
| `crates/cclab-jet/src/transform/mod.rs` | Modify | Expose strip_types and transform_jsx as public API for JIT engine |
| `crates/cclab-jet/src/resolver/mod.rs` | Modify | Support tsconfig.json paths resolution for JIT imports |
| `crates/cclab-jet/Cargo.toml` | Modify | Add `notify` (file watcher), `tempfile` dependencies |

### Milestone 3: Task Runner

| File | Action | Description |
|------|--------|-------------|
| `crates/cclab-jet/src/task_runner/mod.rs` | Create | TaskRunner: load jet.config.yaml, orchestrate parallel execution |
| `crates/cclab-jet/src/task_runner/graph.rs` | Create | TaskGraph: DAG construction, topological sort, cycle detection, ready-task scheduling |
| `crates/cclab-jet/src/task_runner/config.rs` | Create | JetConfig parser: read jet.config.yaml pipeline section, TaskDef deserialization |
| `crates/cclab-jet/src/cli.rs` | Modify | Wire task runner into `jet run` when task name matches pipeline config |
| `crates/cclab-jet/src/lib.rs` | Modify | Export task_runner module |

### Milestone 4: Task Caching

| File | Action | Description |
|------|--------|-------------|
| `crates/cclab-jet/src/task_runner/cache.rs` | Create | TaskCache: content-hash computation (SHA-256 of inputs+env+cmd+deps), local cache store/restore |
| `crates/cclab-jet/src/task_runner/hash.rs` | Create | Hash computation: glob input files, hash env vars, combine with command and dependency hashes |
| `crates/cclab-jet/src/cli.rs` | Modify | Add `--dry` flag, `--filter` flag for task runner commands |
| `crates/cclab-jet/Cargo.toml` | Modify | Add `sha2` (hashing), `serde_yaml` (config parsing) dependencies |

### Integration with Workspace

| File | Action | Description |
|------|--------|-------------|
| `crates/cclab-jet/src/task_runner/workspace.rs` | Create | Workspace-aware task scheduling: cross-package deps (^task), filter by package pattern |
| `crates/cclab-jet/src/pkg_manager/workspace.rs` | Modify | Expose workspace graph for task runner cross-package dependency resolution |
# Reviews
