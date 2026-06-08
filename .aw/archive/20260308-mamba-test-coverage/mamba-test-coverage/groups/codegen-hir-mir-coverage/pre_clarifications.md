---
change: mamba-test-coverage
group: codegen-hir-mir-coverage
date: 2026-03-08
status: answered
---

# Pre-Clarifications

### Q1: llvm-codegen-testability
- **Answer**: LLVM codegen does NOT require LLVM installation. codegen/llvm.rs (531 lines) generates textual LLVM IR and optionally calls system `llc` for object file emission. No llvm-sys dependency in Cargo.toml. Can be tested independently by verifying generated IR text output. If `llc` is unavailable, it falls back to returning IR text.

### Q2: hir-mir-size
- **Answer**: HIR (hir/mod.rs, 227 lines) and MIR (mir/mod.rs, 143 lines) are full implementations, NOT stubs. Tarpaulin only reported 14 and 4 coverable lines because most of the code is type/enum definitions (not executable). HIR has 15+ statement types and 20+ expression types. MIR has CFG-based representation with 15 instruction types. The low coverable count means there's very little executable code to test — mostly data structures. Name resolution is in resolve/pass.rs (420 lines) + resolve/scope.rs (108 lines), which is the real testing target for #747.

