---
number: 804
title: "feat(lens): TypeScript/Rust type inference — close generic and trait gaps"
state: open
labels: [enhancement, P2, crate:lens]
group: "type-inference-gaps"
---

# #804 — feat(lens): TypeScript/Rust type inference — close generic and trait gaps

## Context

Type inference for TypeScript and Rust has known gaps marked with TODO in the codebase.

## TypeScript gaps (`types/ts_infer.rs`)
- Generic type argument parsing incomplete (~line 1290)
- Mapped types not resolved
- Conditional types (`T extends U ? X : Y`) not handled
- Template literal types not supported

## Rust gaps (`types/rust_infer.rs`)
- Size expressions in array types not parsed (`[T; N]` where N is const expr)
- Complex trait bounds (`T: Fn(A) -> B + Send + 'static`)
- Associated type projections (`<T as Trait>::Item`)
- Lifetime inference (currently ignored)

## Impact
- Hover info shows `unknown` for complex generic types
- Go-to-definition may miss impl blocks with complex bounds
- MCP `lens_type_at` returns incomplete info

## Files to modify
- `crates/cclab-lens/src/types/ts_infer.rs`
- `crates/cclab-lens/src/types/rust_infer.rs`
