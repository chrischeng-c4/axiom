---
change: mamba-conformance-basics
group: runtime-basics
date: 2026-03-23
status: answered
---

# Pre-Clarifications

### Q1: Scope boundary
- **Answer**: Fix all 3 bugs in this change. They're all basic runtime correctness — can't ship anything else until these work.

### Q2: Recursion bug root cause
- **Answer**: Likely in codegen — recursive call result not properly returned. Check Cranelift JIT function call emission: the return value from recursive mb_call may not be propagated back through NaN-boxing. Debug by tracing fib(3) step by step.

### Q3: String concat dispatch
- **Answer**: String + string should call mb_str_concat in string_ops.rs. The type checker wrongly requires numeric types for +. Fix in types/check_expr.rs or codegen binary op emission to dispatch to concat for str operands.

### Q4: print() return value
- **Answer**: print() should return None (TAG_NONE). If it returns 0 (TAG_INT), the top-level expression evaluator is printing the return value. Fix: either make print return None, or suppress top-level expression result printing (REPL vs script mode).

