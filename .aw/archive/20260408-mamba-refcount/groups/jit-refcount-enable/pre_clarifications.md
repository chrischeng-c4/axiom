---
change: mamba-refcount
group: jit-refcount-enable
date: 2026-04-08
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should mb_closure_release also release values in the defaults Vec, or are default argument values always NaN-boxed scalars that don't need releasing?
- **Answer**: Yes, release defaults too. Python default args can be arbitrary objects including mutable containers (e.g., `def f(x=[1,2,3])`). The closure owns these values and must cascade-release them. Confirmed by reading MbClosure struct at closure.rs:14-28 — defaults is Vec<MbValue> with no ownership transfer on access.

### Q2: General
- **Question**: After enabling EMIT_REFCOUNT_CALLS, should we also add mb_closure_release calls at function return for closure locals, or are closures already handled by the return cleanup logic?
- **Answer**: No additional change needed. Closures use integer handles (MbValue::from_int(id)) stored in thread-local HashMap, not heap pointers. The JIT return cleanup calls mb_release_value on all locals, which is a no-op for integer handles. Explicit mb_closure_release calls are already emitted by the compiler where needed. The fix is only in mb_closure_release itself — making it cascade-release captured values when the closure is destroyed.

### Q3: General
- **Question**: Should we run the conformance suite under ASan as part of this change, or defer ASan validation to a follow-up?
- **Answer**: Defer ASan to manual verification. Primary validation: (1) jit_refcount_audit_tests pass with --test-threads=1, (2) full conformance suite passes, (3) no SIGBUS/SIGSEGV in any test. ASan requires nightly Rust toolchain and is a belt-and-suspenders check.

