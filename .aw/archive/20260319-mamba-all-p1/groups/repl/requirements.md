---
change: mamba-all-p1
group: repl
date: 2026-03-19
---

# Requirements

Implement an interactive REPL (Read-Eval-Print Loop) for Mamba:
- Entry point: `mamba` with no arguments launches REPL shell
- Prompt: `>>>` for new statements, `...` for continuation lines
- State persistence: variable/function bindings accumulate across inputs
- Multi-line input detection: detect incomplete statements by indent/open brackets
- JIT execution: reuse existing JIT backend to execute expressions and statements
- Pretty printing: display expression return values (with optional syntax highlighting)
- Optional: tab completion (rustyline), command history (up/down navigation)
Acceptance: basic REPL session (variable assignment, expression evaluation, function definition and call) works; multi-line input (function bodies) is correctly handled.
