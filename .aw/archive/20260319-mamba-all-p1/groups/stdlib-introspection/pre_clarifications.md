---
change: mamba-all-p1
group: stdlib-introspection
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: AST types are pub in Rust (parser/ast.rs: pub struct Module, pub enum Stmt with 20+ variants, pub enum Expr with 30+ variants). Exposed via pub mod ast in lib.rs. However, NOT exposed to Python runtime — strictly compile-time internal. No stabilization needed for Rust consumers; need new Python-facing wrapper layer.

### Q2: General
- **Answer**: No ast module exists yet. Internal AST is Mamba-specific (different field names, different node structure from CPython). Recommend presenting CPython-compatible AST with same field names and node types to enable ast.NodeVisitor compatibility. Map internal nodes to CPython-equivalent structures at the module boundary.

### Q3: General
- **Answer**: No dis module exists. Mamba uses MIR (mir/mod.rs with MirBody, BasicBlock, MirInst, Terminator) + Cranelift for codegen. Recommend exposing MIR-level instructions for debugging Mamba internals. Full CPython bytecode compatibility is not feasible since Mamba has no bytecode VM. Use case is primarily debugging.

