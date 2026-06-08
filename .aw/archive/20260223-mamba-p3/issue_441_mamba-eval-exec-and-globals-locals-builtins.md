---
number: 441
title: "mamba: eval/exec and globals/locals builtins"
state: open
labels: [enhancement, crate:mamba, P3]
---

# #441 — mamba: eval/exec and globals/locals builtins

## Description

Implement dynamic code execution and scope introspection builtins.

## Requirements

### Dynamic execution
- R1: `eval(expression, globals=None, locals=None)` — evaluate expression string
- R2: `exec(code, globals=None, locals=None)` — execute code string
- R3: `compile(source, filename, mode)` — compile to code object

### Scope introspection
- R4: `globals()` — return current global scope dict
- R5: `locals()` — return current local scope dict

## Notes

These are architecturally challenging for a compiled language. Options:
1. Embed the parser/interpreter for eval/exec
2. Support only constant expressions in eval
3. Implement globals/locals via runtime scope objects

Many Python programs work without eval/exec, but some metaprogramming patterns depend on it.

## Priority

P3 — architecturally complex; many programs don't need dynamic execution.
