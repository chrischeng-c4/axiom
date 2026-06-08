---
number: 1089
title: "jet dev: implement optimizeDeps — full CJS→ESM pre-bundling for node_modules"
state: open
labels: [type:enhancement, priority:p1, crate:jet]
group: "jet-dev-server-v2"
---

# #1089 — jet dev: implement optimizeDeps — full CJS→ESM pre-bundling for node_modules

## Problem

Jet dev server cannot fully serve React apps because many npm packages (react, react-dom, axios, date-fns) ship CommonJS format. Browsers only understand ESM. The current pre-bundling approach (line-based `require()` flattening + ESM wrapper) is fragile:

1. `require()` inside conditionals (`if (process.env.NODE_ENV === 'production')`) causes both branches to be inlined
2. Cross-package `require()` (react-dom → react, scheduler) needs correct load ordering via `window.__jetRequireCache` which is timing-dependent
3. Packages with complex require chains (axios → form-data → Node.js builtins) fail completely

## Success Criteria

1. `cclab jet dev` on a fresh `create-react-app` or Vite-template project renders without errors
2. All CJS dependencies are pre-bundled into individual ESM files in `node_modules/.jet/`
3. Pre-bundling runs once on startup, cached until `package.json` or lock file changes
4. `react`, `react-dom`, `react-dom/client`, `react/jsx-runtime`, `axios`, `date-fns` all work
5. No `Unexpected token` errors from CJS syntax in browser
6. Startup time < 3s for typical React project (~20 CJS deps)

## Boundary Conditions

- Packages with `"module"` or `"exports.import"` field → skip (already ESM)
- Packages with Node.js native addons (`.node` files) → skip with warning
- Circular `require()` between packages → detect and break cycle
- `process.env.NODE_ENV` conditionals → resolve to `'development'` in dev mode
- Subpath exports (`react/jsx-runtime`) → must also be pre-bundled
- Transitive CJS deps (not in direct `dependencies`) → auto-discovered and bundled
- Scoped packages (`@tanstack/react-query`) → handled correctly
- `package.json` `"exports"` map with conditions (`"import"`, `"require"`, `"default"`) → resolve correctly

## Implementation Approach

Replace line-based flattening with Jet's own bundler per-dependency:
1. Scan `package.json` dependencies
2. For each CJS dep, create virtual entry: `export * from 'dep'; export { default } from 'dep';`
3. Use `crate::resolver` to resolve the actual file paths
4. Bundle with `crate::bundler::Bundler` (handles require→import, tree-shaking)
5. Write to `node_modules/.jet/{name}.mjs`
6. Update importmap to point to `.jet/` cache

## Current State

Partial implementation exists in `crates/cclab-jet/src/dev_server/mod.rs`:
- `pre_bundle_cjs_deps()` — line-based flattening
- `flatten_cjs()` — recursive require inlining
- `extract_require_path()` — require path extraction
- `discover_transitive_cjs_deps()` — transitive dep scanning

These should be replaced with bundler-based approach.

## Test Cases

```
# Must pass:
curl localhost:3000/node_modules/.jet/react.mjs | node -e "import('/dev/stdin')"  # valid ESM
curl localhost:3000/node_modules/.jet/react-dom.mjs | grep "import.*react"  # has dep import
ls node_modules/.jet/ | wc -l  # > 0 after startup
# Conductor frontend renders dashboard with 0 console errors
```
