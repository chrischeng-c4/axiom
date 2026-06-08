---
change: mamba-conformance-basics
group: runtime-basics
date: 2026-03-23
---

# Requirements

Fix three critical correctness bugs in the Mamba runtime/codegen that block any meaningful conformance testing: (1) recursive functions return the wrong value — fib(30) returns 0 instead of 832040, indicating a broken return-value propagation path through recursive call frames; (2) string concatenation via the + operator raises a type error instead of concatenating, indicating missing or broken str.__add__ dispatch in the operator codegen; (3) print() returns 0 instead of None, and this spurious 0 leaks into stdout as extra output, indicating the JIT treats the void/None return of print as integer 0. All three are in cclab-mamba (runtime, codegen, or builtins layer). These must be fixed before the broader Py3.12 conformance testing suite described in #1037 can produce meaningful results.
