---
change: jet-remaining
group: bundle-optimization-hoisting
date: 2026-03-19
---

# Requirements

Optimize bundle size by implementing advanced scope hoisting and module flattening in scope_hoist.rs. Phase 1 involves module concatenation for non-circular modules to remove wrapper functions. Phase 2 moves to 'true' module flattening where module bodies are merged into a single function scope with prefixed renaming. Implement cross-module constant inlining and unified dead code elimination (DCE) to achieve a bundle size competitive with Vite/Webpack (target ≤ 196KB for react-bench).
