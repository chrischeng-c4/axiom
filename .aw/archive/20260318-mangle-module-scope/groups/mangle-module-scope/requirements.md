---
change: mangle-module-scope
group: mangle-module-scope
date: 2026-03-18
---

# Requirements

Fix the jet bundler to close the ~15KB bundle size gap (currently 215KB vs webpack 192KB) by enabling the minifier to rename variables that live inside per-module IIFE wrappers. Two tightly coupled changes are required: (1) Extend `mangle.rs` so the mangler descends into per-module `!function(module,exports,require){...}()` wrapper scopes and renames their top-level identifiers — identifiers like `workInProgress` appear 267× and alone account for ~3.5KB of savings. (2) Implement true module flattening in `scope_hoist.rs`: merge all per-module IIFE wrappers into a single flat function scope, rename each module's top-level variables with a collision-avoiding prefix (`_m0_foo`, `_m1_bar`), and replace `require(N)` calls with direct references to the renamed export variables. Skip flattening for modules that use `eval()`, `with`, or dynamic `arguments` access. Optionally include cross-module constant inlining (inline unreassigned `const` at import sites) and cross-module DCE (remove functions only reachable from already-eliminated dead code) as follow-on sub-phases. Success criteria: react-bench bundle ≤ 196KB, all 126 bundler tests pass, mini-react Playwright tests pass on both Vite and Jet builds.
