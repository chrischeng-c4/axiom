---
number: 764
title: "feat(jet): JIT execution + task runner — script runner, task graph, caching"
state: open
labels: [enhancement, P1, crate:jet]
group: "jit-runner"
---

# #764 — feat(jet): JIT execution + task runner — script runner, task graph, caching

## Goal

Add JIT execution (run JS/TS without pre-build) and task runner (parallel script orchestration with caching), comparable to Bun run + Turborepo.

## Milestone 1: Script Runner

| Feature | Description |
|---------|-------------|
| **jet run \<script\>** | Execute package.json scripts with `.bin` on PATH |
| **jet exec \<cmd\>** | Run arbitrary command with `node_modules/.bin` on PATH |
| **jet dlx \<pkg\>** | Download and execute a package (like `npx`) |
| **Lifecycle ordering** | pre/post script hooks (pretest → test → posttest) |
| **Env injection** | `NODE_ENV`, `JET_*` variables |

## Milestone 2: JIT TypeScript/JSX Execution

| Feature | Description |
|---------|-------------|
| **jet run file.ts** | Execute TypeScript files directly (no tsc step) |
| **jet run file.tsx** | Execute TSX/JSX files directly |
| **Module loader** | Custom ESM loader that transforms on-the-fly |
| **Source maps** | Inline source maps for accurate stack traces |
| **Import resolution** | `tsconfig.json` paths, `package.json` exports |
| **Watch mode** | `jet run --watch file.ts` — restart on changes |

### Implementation approach

```
Source file → Tree-sitter transform (strip types, transform JSX)
           → Write to temp .js
           → Execute via Node.js child process
           → Clean up
```

Alternative (advanced): embed V8/QuickJS for direct execution without Node.js dependency.

## Milestone 3: Task Runner

| Feature | Description |
|---------|-------------|
| **Task graph** | Define task dependencies in `jet.config.yaml` |
| **Parallel execution** | Run independent tasks concurrently |
| **Topological ordering** | Respect inter-task dependencies |
| **Content-hash caching** | Skip tasks when inputs haven't changed |
| **Output caching** | Cache and restore task outputs (build artifacts) |
| **Filtering** | `jet run build --filter=pkg-a` |

### Task config format

```yaml
# jet.config.yaml
pipeline:
  build:
    dependsOn: ["^build"]
    outputs: ["dist/**"]
    inputs: ["src/**", "package.json"]
  test:
    dependsOn: ["build"]
    outputs: []
  lint:
    outputs: []
  dev:
    cache: false
    persistent: true
```

## Milestone 4: Advanced Caching

| Feature | Description |
|---------|-------------|
| **Local cache** | `~/.jet-cache/tasks/` content-addressed |
| **Remote cache** | HTTP-based shared cache (S3, GCS, custom) |
| **Cache key** | Hash of: input files, env vars, command, deps outputs |
| **Replay** | Restore stdout/stderr from cache on hit |
| **Dry run** | `jet run build --dry` — show what would run |

## Architecture Notes

```
jet run <script>
  ├─ Check package.json scripts → found? exec via sh -c
  ├─ Check if file exists → found? JIT execute
  └─ Not found → error

jet run build (task runner mode)
  ├─ Read jet.config.yaml pipeline
  ├─ Build task dependency graph
  ├─ Compute content hashes for each task
  ├─ Check cache → hit? replay outputs
  └─ Miss? execute, capture outputs, store cache
```

## Acceptance Criteria

- [ ] `jet run test` executes package.json `test` script
- [ ] `jet run file.ts` executes TypeScript without compilation step
- [ ] Task graph respects `dependsOn` ordering
- [ ] Cached tasks skip execution and replay outputs
- [ ] `--watch` mode re-runs on file changes
