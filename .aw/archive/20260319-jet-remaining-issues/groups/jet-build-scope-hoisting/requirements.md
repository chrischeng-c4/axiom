---
change: jet-remaining-issues
group: jet-build-scope-hoisting
date: 2026-03-19
---

# Requirements

Implement scope hoisting in two phases to close the bundle size gap (215KB jet vs 192KB webpack on react-bench). Phase 1 (#882): module concatenation — for non-circular modules with a single import site, inline the module body directly at the import site and remove the __jet__.define/__jet__.require wrapper; preserve wrapped form for circular deps and dynamic imports. Phase 2 (#903 — true module flattening): merge all flattenable module bodies into a single function scope with module-prefixed variable renaming (_m0_foo, _m1_bar); replace require(N) calls with direct variable references; skip modules containing eval(), with-statements, or arguments references. Follow with Phase 2b (cross-module constant inlining: identify never-reassigned const bindings, inline at import sites across module boundaries, re-run DCE) and Phase 2c (cross-module DCE: used-export analysis on unified scope, remove functions/classes only referenced by dead code, preserve side-effectful code). Update scope_hoist.rs (currently 209L) and mangle.rs for unified-scope variable mangling. Success criteria: react-bench bundle ≤ 196KB, all 126 bundler tests pass, mini-react Playwright tests pass on both Vite and Jet builds.
