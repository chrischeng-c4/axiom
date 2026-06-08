---
change: mamba-conformance-p0
group: mamba-p0-fixes
date: 2026-03-24
status: answered
---

# Pre-Clarifications

### Q1: fix-approach
- **Answer**: All 6 fixes target the Cranelift JIT codegen and scope resolution. Lambda/with/decorator SIGBUS are all calling convention issues in codegen/cranelift/mod.rs. Stdlib None returns is a module Dict dispatch issue in runtime/class.rs. Comprehension scope and walrus are in resolve/pass.rs. Each fix is independent — no ordering dependency.

