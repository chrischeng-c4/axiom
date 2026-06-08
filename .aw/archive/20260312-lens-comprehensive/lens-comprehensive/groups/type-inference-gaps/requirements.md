---
change: lens-comprehensive
group: type-inference-gaps
date: 2026-03-12
---

# Requirements

Close known gaps in TypeScript and Rust type inference:

TypeScript (types/ts_infer.rs):
- Generic type argument parsing (TODO at ~line 1290)
- Mapped types resolution
- Conditional types (T extends U ? X : Y)
- Template literal types

Rust (types/rust_infer.rs):
- Size expressions in array types [T; N]
- Complex trait bounds (T: Fn(A) -> B + Send + 'static)
- Associated type projections (<T as Trait>::Item)
- Lifetime inference (currently ignored)

Fix hover info showing 'unknown' for complex generic types. Ensure MCP lens_type_at returns complete info.
