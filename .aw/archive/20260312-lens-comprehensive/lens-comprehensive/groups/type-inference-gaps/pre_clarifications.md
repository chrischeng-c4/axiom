---
change: lens-comprehensive
group: type-inference-gaps
date: 2026-03-12
status: answered
---

# Pre-Clarifications

### Q1: Lifetime inference
- **Answer**: Implement full Rust lifetime elision rules. Infer omitted lifetimes per Rust reference rules (single input lifetime, &self, etc.).

### Q2: Conditional types
- **Answer**: Full resolution: when T is known, resolve T extends U ? X : Y to X or Y. When unknown, display the conditional expression.

