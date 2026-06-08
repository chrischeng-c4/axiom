---
number: 1120
title: "jet build: scope hoisting (module concatenation)"
state: open
labels: [type:enhancement, priority:p2, crate:jet]
group: "scope-hoisting"
---

# #1120 — jet build: scope hoisting (module concatenation)

## Problem

Jet AOT build produces ~206.8 KB for a React benchmark vs Vite's ~192 KB (~7.7% larger). The main cause is that each module is wrapped in a separate scope, adding overhead. Scope hoisting (module concatenation) inlines small modules into their parent, reducing wrapper overhead and enabling cross-module dead code elimination.

## Success Criteria

1. React benchmark bundle size ≤ 195 KB (within 2% of Vite)
2. Modules with single importer are inlined (no separate wrapper)
3. Side-effect-free modules (per `package.json` `sideEffects: false`) are candidates for inlining
4. Circular dependencies are excluded from inlining (remain wrapped)
5. No behavior change — all existing tests still pass

## Boundary Conditions

- Modules with side effects → never inline
- Modules imported by 2+ parents → keep as shared chunk (already works)
- Circular dependency groups → exclude from concatenation
- Dynamic `import()` targets → always separate chunk (already works)
- CSS modules → not affected (separate pipeline)

## Current State

Draft spec exists in `cclab/specs/crates/cclab-jet/scope-hoisting.md`. Implementation scaffold exists in `bundler/scope_hoist.rs` but is incomplete.
