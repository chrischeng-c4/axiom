---
number: 1092
title: "jet install: .jet-store symlinks break Node.js module resolution for native deps"
state: open
labels: [type:bug, priority:p1, crate:jet]
group: "jet-dev-server-v2"
---

# #1092 — jet install: .jet-store symlinks break Node.js module resolution for native deps

## Problem

Jet's package manager installs packages into a flat `~/.jet-store/{name}@{version}/` directory and symlinks into `node_modules/`. This breaks Node.js module resolution when:

1. Package A (in .jet-store/A@1.0/) imports package B — Node resolves relative to `.jet-store/A@1.0/`, not `node_modules/`, so B is not found
2. Packages with `optionalDependencies` (esbuild, rollup) need platform-specific native binaries as siblings — `.jet-store` doesn't place them correctly

**Impact:** Vite cannot run when installed via `cclab jet install` because `vite` → `esbuild` resolution fails:
```
Error [ERR_MODULE_NOT_FOUND]: Cannot find package 'esbuild' imported from /Users/.../.jet-store/vite@5.4.21/dist/node/cli.js
```

Same for rollup → `@rollup/rollup-darwin-arm64`.

## Success Criteria

1. `cclab jet install` + `npx vite` works without errors
2. `cclab jet install` + `node -e "require('esbuild')"` works
3. Packages with optionalDependencies install their platform binaries correctly
4. Node.js `require()` and `import` resolution works from any package in node_modules

## Boundary Conditions

- Regular packages without native deps → current symlink approach is fine
- Packages with `optionalDependencies` → must install native binaries alongside
- Packages importing each other (A→B) → B must be resolvable from A's install location
- ESM `import` resolution → follows different algorithm than CJS `require()`
- `peerDependencies` → must be resolvable from the dependent package
- Monorepo workspace packages (`workspace:*`) → already handled via workspace protocol

## Reproduction

```bash
cd projects/conductor/fe
cclab jet install
npx vite  # → ERR_MODULE_NOT_FOUND: Cannot find package 'esbuild'
node -e "require('vite')"  # → same error
```

## Workaround

```bash
# Use pnpm instead of jet for install
pnpm install --no-frozen-lockfile
npx vite  # works
```

## Root Cause

`.jet-store` uses flat hardlinks — each package is isolated with no awareness of its dependencies. Node.js module resolution walks up the directory tree from the package's location, but `.jet-store/vite@5.4.21/node_modules/` doesn't exist.

**Options:**
1. Create nested `node_modules` within each store entry for its deps (like pnpm)
2. Use `.pnp.cjs` style resolution hook
3. For native/optional deps, always install directly into project `node_modules/` (not store)
