---
change: jet-jit-runner
group: jit-runner
date: 2026-03-11
status: answered
---

# Pre-Clarifications

### Q1: JIT execution engine
- **Answer**: Node.js child process with Tree-sitter transform. Reuse existing transform module (strip types, transform JSX) → write temp .js → execute via `node` child process → cleanup. Similar to tsx approach. Simplest and most compatible.

### Q2: Task config location
- **Answer**: Dedicated `jet.config.yaml` file with `pipeline` section. Similar to Turborepo's turbo.json approach.

### Q3: Remote cache scope
- **Answer**: Deferred to future issue. This change only implements local cache at `~/.jet-cache/tasks/`. Remote cache (S3/GCS/HTTP) will be a separate issue.

### Q4: Workspace integration
- **Answer**: Yes, integrate with workspace module from jet-pnpm-parity. Support `^build` cross-package dependencies in task graph. Use workspace topological ordering for cross-package task execution.

