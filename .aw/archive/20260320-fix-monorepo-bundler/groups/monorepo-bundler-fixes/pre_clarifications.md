---
change: fix-monorepo-bundler
group: monorepo-bundler-fixes
date: 2026-03-20
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: (b) workspace root — stop at the directory containing nx.json or the top-level package.json. WorkspaceMode::detect() already finds this. Reuse that.

### Q2: General
- **Answer**: Use the existing __jet__ runtime format (generate_bundle_with_runtime in mod.rs). Cyclic modules stay as __jet__.define(N, function(){...}) with __jet__.require(N). Non-cyclic modules use scope hoisting. This is the existing Phase 1 IIFE wrapper — just don't bail on the entire bundle when a cycle is found.

### Q3: General
- **Answer**: (b) build completes AND React is included in the bundle (output > 100KB for demo app). Add integration test with trimmed fixture.

