---
number: 962
title: "jet-build: bundler resolver doesn't walk up to monorepo root node_modules"
state: open
labels: [bug, P1, crate:jet]
group: "monorepo-bundler-fixes"
---

# #962 — jet-build: bundler resolver doesn't walk up to monorepo root node_modules

## Problem

In Nx monorepos, dependencies are installed at the workspace root (`frontend/node_modules/`), but projects live in subdirectories (`apps/demo/`). Jet's bundler resolves imports from the project dir and doesn't walk up parent directories to find `node_modules`.

Result: `import React from 'react'` is treated as external (8.7KB output instead of 140KB with React bundled).

## Expected

Node.js resolution algorithm: look for `node_modules/react` in current dir, then parent, then grandparent, etc.

## Also fix

Circular dependency in `shared-ui-form-inputs` — keep module wrappers for cyclic modules instead of bailing.

## References
- `crates/cclab-jet/src/resolver/` — module resolver
- Discovered on tech-platform Nx monorepo
