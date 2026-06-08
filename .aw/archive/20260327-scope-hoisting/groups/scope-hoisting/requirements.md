---
change: scope-hoisting
group: scope-hoisting
date: 2026-03-26
---

# Requirements

Implement scope hoisting (module concatenation) in jet AOT build to reduce bundle size from ~206.8 KB to ≤195 KB (within 2% of Vite's ~192 KB).

1. Inline modules with single importer into their parent (no separate wrapper function)
2. Use `sideEffects: false` from package.json to identify inlining candidates
3. Exclude circular dependency groups from concatenation
4. Exclude dynamic import() targets (already separate chunks)
5. Exclude modules with side effects
6. Enable cross-module dead code elimination after inlining
7. Existing draft spec at cclab/specs/crates/cclab-jet/scope-hoisting.md and scaffold at bundler/scope_hoist.rs
8. All existing tests must still pass — no behavior change
