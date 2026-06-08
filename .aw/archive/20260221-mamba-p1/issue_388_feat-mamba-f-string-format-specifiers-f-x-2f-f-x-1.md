---
number: 388
title: "feat(mamba): f-string format specifiers (f\"{x:.2f}\", f\"{x:>10}\")"
state: open
labels: [enhancement, P1, crate:mamba]
---

# #388 — feat(mamba): f-string format specifiers (f"{x:.2f}", f"{x:>10}")

## Summary
Support Python format specifiers inside f-string interpolation expressions.

## Required
- Numeric formatting: `f"{x:.2f}"`, `f"{x:,}"`, `f"{x:010d}"`
- Alignment: `f"{x:<10}"`, `f"{x:>10}"`, `f"{x:^10}"`
- Fill character: `f"{x:*>10}"`
- Type codes: `d`, `f`, `e`, `g`, `x`, `o`, `b`, `s`, `c`, `%`
- Width and precision: `f"{x:10.2f}"`
- Sign: `f"{x:+}"`, `f"{x: }"`
- Nested expressions in format spec: `f"{x:{width}.{precision}f}"`

## Implementation Notes
- Lexer/parser need to parse format spec after `:` inside `{}`
- Runtime `mb_string_format` needs to handle format spec string
- Can leverage Rust's `format!` machinery for the heavy lifting
