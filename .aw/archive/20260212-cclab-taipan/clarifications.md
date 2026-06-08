---
change: cclab-taipan
date: 2026-02-12
---

# Clarifications

## Q1: Git Workflow
- **Question**: Which git workflow should we use for this change?
- **Answer**: in_place — work on the current branch (feat/taipan)
- **Rationale**: The feat/taipan branch already exists and is the natural place for this work.

## Q2: Codegen Backends
- **Question**: For v0.1, which codegen backends should be fully implemented?
- **Answer**: Cranelift only. LLVM and WASM backends will be stubbed with trait implementations but no real codegen.
- **Rationale**: Cranelift is pure Rust, easiest to integrate, and sufficient for v0.1. Stubbing LLVM/WASM keeps the architecture extensible without the implementation burden.

## Q3: Standard Library Scope
- **Question**: Should the v0.1 scope include a standard library or just compiler infrastructure with minimal builtins?
- **Answer**: Minimal builtins only — print() and basic operators. No stdlib.
- **Rationale**: Focus on compiler correctness and pipeline completeness. A stdlib can be added incrementally later.

## Q4: Change Scope
- **Question**: Should we implement the full plan in one change or split into multiple sequential changes?
- **Answer**: Single change covering the full compiler pipeline end-to-end.
- **Rationale**: Keeps everything cohesive. The compiler pipeline is interdependent — splitting would create partial, untestable states.

