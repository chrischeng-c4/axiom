---
change: fix-conformance-xfails
group: mamba-conformance-xfails
date: 2026-03-23
status: answered
---

# Pre-Clarifications

### Q1: Fix scope
- **Answer**: Address all xfail categories in one change. All 31 xfails should be fixed or have their fixture simplified to test only what works. ExceptionGroup/except* and async generators can remain xfailed as genuinely unimplemented features. The goal is to maximize passing conformance tests.

### Q2: Codegen IR root cause
- **Answer**: The codegen IR issues are calling-convention mismatches. classmethod and descriptor __get__ emit function signatures with wrong parameter counts. getattr/setattr/delattr emit invalid Cranelift IR (verifier fails). super() emits duplicate function definitions (same symbol defined twice). These are in cranelift/mod.rs IR generation for class method dispatch.

### Q3: Stdlib non-iterable returns
- **Answer**: All stdlib functions return None — this is a single shared issue. Module attribute access (e.g., json.dumps) resolves the function but the return value from the call is not being propagated back. The root cause is likely in how module function calls are lowered in hir_to_mir.rs or how the codegen handles CallExtern results for module-level functions. This is one fix, not six.

### Q4: Parser nested f-string approach
- **Answer**: The Mamba parser is hand-written recursive-descent (parser/expr_compound.rs). Nested f-strings require lexer-level changes to support re-entrant f-string tokenization. The current lexer tracks a single f-string nesting level.

### Q5: Walrus scope fix layer
- **Answer**: The walrus scope bug is in the symbol-table construction pass (resolve/pass.rs). The := operator in comprehensions should assign to the enclosing function scope, not the comprehension's own scope. The resolution pass incorrectly creates the binding in the innermost scope.

