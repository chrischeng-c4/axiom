---
change: jet-jit-runner
group: jit-runner
date: 2026-03-11
---

# Requirements

Add JIT execution and task runner to cclab-jet, comparable to Bun run + Turborepo.

**Script Runner (M1):**
- `jet run <script>`: execute package.json scripts with `.bin` on PATH
- `jet exec <cmd>`: run arbitrary command with node_modules/.bin on PATH
- `jet dlx <pkg>`: download + execute a package (like npx)
- Lifecycle ordering: pre/post script hooks (pretest → test → posttest)
- Env injection: NODE_ENV, JET_* variables

**JIT TypeScript/JSX Execution (M2):**
- `jet run file.ts`: execute TypeScript files directly without tsc
- `jet run file.tsx`: execute TSX/JSX files directly
- Tree-sitter transform → write temp .js → execute via Node.js child process → cleanup
- Inline source maps for accurate stack traces
- Import resolution: tsconfig.json paths, package.json exports
- Watch mode: `jet run --watch file.ts` — restart on changes

**Task Runner (M3):**
- Task graph defined in `jet.config.yaml` with `pipeline` section
- `dependsOn` for inter-task dependencies, `^build` for cross-package deps
- Parallel execution of independent tasks via topological ordering
- Content-hash caching: skip tasks when inputs haven't changed
- Output caching: cache and restore task outputs (dist/**)
- Filtering: `jet run build --filter=pkg-a`

**Advanced Caching (M4):**
- Local cache at `~/.jet-cache/tasks/` content-addressed
- Cache key = hash of input files + env vars + command + deps outputs
- Replay stdout/stderr from cache on hit
- Dry run: `jet run build --dry` — show what would run
- Remote cache deferred to future (HTTP-based S3/GCS)

**Acceptance Criteria:**
- `jet run test` executes package.json test script
- `jet run file.ts` executes TypeScript without compilation step
- Task graph respects dependsOn ordering
- Cached tasks skip execution and replay outputs
- --watch mode re-runs on file changes
