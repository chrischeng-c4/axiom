---
number: 838
title: "REPL — interactive Mamba shell"
state: open
labels: [enhancement, P1, crate:mamba]
group: "repl"
---

# #838 — REPL — interactive Mamba shell

## Summary

Implement an interactive REPL (Read-Eval-Print Loop) for Mamba, similar to the Python interactive interpreter.

```
$ mamba
Mamba 0.3.34 (force-typed Python, Cranelift JIT)
>>> x = 42
>>> x * 2
84
>>> def greet(name: str) -> str:
...     return f"Hello, {name}!"
>>> greet("world")
'Hello, world!'
```

## Why P1

A REPL is essential for:
- Language exploration and learning
- Quick prototyping
- Demonstrating Mamba's capabilities
- Developer adoption (people expect `python` → `mamba` drop-in)

## Scope

- **JIT backend**: Already exists, can execute single expressions
- **State persistence**: Accumulate bindings across REPL inputs
- **Multi-line input**: Detect incomplete statements (indent continuation)
- **Tab completion**: Optional, via readline/rustyline
- **History**: Command history with up/down navigation
- **Pretty printing**: Format return values with syntax highlighting
