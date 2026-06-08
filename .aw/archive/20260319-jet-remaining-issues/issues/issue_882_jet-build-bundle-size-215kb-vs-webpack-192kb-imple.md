---
number: 882
title: "jet-build: bundle size 215KB vs webpack 192KB — implement scope hoisting"
state: open
labels: [enhancement, P1, crate:jet]
group: "jet-build-scope-hoisting"
---

# #882 — jet-build: bundle size 215KB vs webpack 192KB — implement scope hoisting

## Context

React-bench bundle size comparison:

| Bundler | Size | Gap |
|---------|------|-----|
| webpack (Terser) | **192KB** | baseline |
| vite (Rollup+esbuild) | 196KB | +2% |
| **jet** | **215KB** | **+12%** |

Build speed: jet wins (1.1s vs vite 1.3s vs webpack 2.5s wall clock).

## Root Cause

Jet wraps each module in `__jet__.define(id, function(require, module, exports) { ... })`. This per-module wrapper adds:
- ~50 bytes overhead per module × ~30 modules = ~1.5KB wrapper overhead
- **No scope hoisting** — modules can't share variable scopes, so:
  - No cross-module dead code elimination
  - No cross-module constant folding
  - Every import/export is a runtime `require()` call = extra code

Webpack/Rollup with scope hoisting (aka "module concatenation") inline modules into a single scope, enabling:
- Terser to eliminate unused cross-module exports
- Smaller variable names (one scope = less mangling collision)
- No wrapper functions or runtime require overhead

## Proposed Implementation

### Phase 1: Module Concatenation (~10KB savings)
- For non-circular, single-import modules: inline the module body directly at the import site
- Remove `__jet__.define` / `__jet__.require` wrapper for inlined modules
- Preserve module boundaries for circular deps and dynamic imports

### Phase 2: Cross-Module DCE (~5KB savings)
- After concatenation, re-run tree shaking on the merged scope
- Eliminate unused exports that are now visible as unused variables

### Phase 3: Unified Minification (~3KB savings)
- Single-pass Terser-quality minification on the concatenated output
- Better variable mangling with full scope visibility

## Target

Bundle size ≤ 196KB (match vite, within 2% of webpack)
