---
number: 903
title: "jet-build: scope hoisting Phase 2 — true module flattening for cross-module optimization"
state: open
labels: [enhancement, P1, crate:jet]
group: "mangle-module-scope"
---

# #903 — jet-build: scope hoisting Phase 2 — true module flattening for cross-module optimization

## Problem

Current scope hoisting (`scope_hoist.rs`, 209L) replaces `__jet__.define/require` with a single IIFE + lightweight `__jet_require` switch, but each module's code still runs inside its own `!function(module,exports,require){...}()` wrapper. This means:

- Minifier cannot reuse short variable names across modules (each has independent scope)
- Cross-module constant inlining is impossible (constants stay inside their function)
- Cross-module DCE cannot eliminate functions only referenced by dead code in other modules

**Result:** react-bench went from 215.0KB → 214.6KB (only -0.4KB), vs target of ≤192KB.

## Proposed Solution

### Phase 2a: True Module Flattening
- Merge module bodies into a single function scope (no per-module wrappers)
- Rename top-level variables with module-prefixed names to avoid collisions (`_m0_foo`, `_m1_bar`)
- Replace `require(N)` calls with direct references to the renamed exports
- Skip flattening for modules with `eval()`, `with`, or `arguments` usage

### Phase 2b: Cross-Module Constant Inlining
- Identify `const` bindings that are never reassigned
- Inline their values at import sites across module boundaries
- Re-run DCE after inlining to eliminate newly-dead branches

### Phase 2c: Cross-Module DCE
- After flattening, run used-export analysis on the unified scope
- Remove functions/classes only referenced by already-eliminated dead code
- Preserve anything with observable side effects

## Success Criteria
- react-bench bundle ≤ 196KB (matching webpack 192KB within ~2%)
- All 126 bundler tests pass
- mini-react Playwright tests pass on both Vite and Jet builds

## References
- `crates/cclab-jet/src/bundler/scope_hoist.rs` — current implementation
- `crates/cclab-jet/src/bundler/mangle.rs` — variable mangling (needs unified scope support)
- #882 (parent issue)
