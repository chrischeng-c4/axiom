---
change: 1129-patrol
group: jit-refcount-audit
date: 2026-04-03
---

# Requirements

Complete the CPython 3.12 reference counting implementation in Mamba's JIT codegen. The retain/release infrastructure is already committed (immortal refcount, JIT wrappers, container retain on store, cascading release, release-before-overwrite, copy retain, return cleanup) but EMIT_REFCOUNT_CALLS flag is disabled due to heap-use-after-free. Remaining work: (1) Audit all mb_* runtime functions to classify return values as new reference (caller owns) or borrowed reference (caller must retain before use), (2) Add mb_retain calls for borrowed-reference returns so callers always get owned references, (3) Enable EMIT_REFCOUNT_CALLS = true, (4) Re-enable GC in gc.rs.
