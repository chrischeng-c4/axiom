---
change: mamba-conformance-p1
group: mamba-p1-oop
date: 2026-03-24
status: answered
---

# Pre-Clarifications

### Q1: approach
- **Answer**: All 5 fixes are in codegen/cranelift/mod.rs and runtime/class.rs. classmethod/property/getattr need correct IR generation. super() return needs MIR lowering fix. MRO needs C3 linearization fix in class registry.

