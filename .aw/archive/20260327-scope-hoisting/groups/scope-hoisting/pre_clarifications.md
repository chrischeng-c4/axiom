---
change: scope-hoisting
group: scope-hoisting
date: 2026-03-26
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should scope hoisting build on the existing scaffold in bundler/scope_hoist.rs?
- **Answer**: Yes. Draft spec at cclab/specs/crates/cclab-jet/scope-hoisting.md and implementation scaffold at bundler/scope_hoist.rs already exist. Build on top of them.

### Q2: General
- **Question**: What is the benchmark target for measuring success?
- **Answer**: Use the react-bench project in examples/react-bench/. Bundle size must be ≤195 KB (within 2% of Vite's ~192 KB). Current jet output is ~206.8 KB.

