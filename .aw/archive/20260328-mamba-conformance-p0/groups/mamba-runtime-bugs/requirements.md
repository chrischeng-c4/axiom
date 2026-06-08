---
change: mamba-conformance-p0
group: mamba-runtime-bugs
date: 2026-03-25
---

# Requirements

Decorated function calls must return the actual return value, not None. 1 // 0 must raise ZeroDivisionError with message 'integer division or modulo by zero'. JIT return convention must be consistent with mb_call0 dynamic dispatch. Cranelift FloorDiv codegen must emit zero-check branch before sdiv.
