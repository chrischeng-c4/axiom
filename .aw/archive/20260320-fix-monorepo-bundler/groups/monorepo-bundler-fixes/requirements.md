---
change: fix-monorepo-bundler
group: monorepo-bundler-fixes
date: 2026-03-20
---

# Requirements

Fix two bugs in the cclab-jet bundler (crates/cclab-jet/src/resolver/) for Nx monorepo compatibility:

1. **Resolver: walk up to root node_modules** — Implement the Node.js module resolution algorithm. When resolving a bare import (e.g., `import React from 'react'`), search for `node_modules/<pkg>` starting from the importing file's directory, then walk up each parent directory until found or filesystem root is reached. Currently the resolver only checks the project dir, causing workspace-root dependencies (e.g., `frontend/node_modules/`) to be treated as external in Nx monorepos (observed: 8.7KB output instead of ~140KB with React bundled).

2. **Circular dependency handling: runtime wrappers instead of bail** — When the bundler detects a circular dependency cycle (as seen in `shared-ui-form-inputs`), instead of bailing/erroring, generate a runtime module wrapper that lazily dereferences the cyclic export. This allows circular graphs to resolve at runtime without build failure.

Acceptance: both fixes must work together on the tech-platform Nx monorepo build.
