---
change: mamba-refcount
group: jit-refcount-enable
date: 2026-04-08
---

# Requirements

Enable CPython 3.12-style reference counting in Mamba's JIT codegen by fixing the remaining closure ownership bug and flipping EMIT_REFCOUNT_CALLS to true. This is a single cohesive change across codegen and runtime modules within cclab-mamba.

### What needs to change

1. **Fix closure ownership symmetry** (root cause of SIGBUS): `mb_closure_release` in `runtime/closure.rs` line 137-140 removes the closure from the thread-local HashMap via `.remove()` but does NOT release the captured MbValues. When refcount calls are enabled, captured heap objects (strings, lists) that were retained via `mb_closure_get_capture` become dangling — the closure's captures are dropped without decrementing refcounts, causing use-after-free when sequential tests share runtime state.

   Fix: Before removing from HashMap, iterate `closure.captures` and call `rc::release_if_ptr(val)` for each captured value. Also release `closure.func` if it's a heap pointer. Also release any values in `closure.defaults`.

2. **Verify all borrowed-reference runtime functions have retain_if_ptr**: The ownership audit (documented in rc.rs lines 1-68) classifies ~22 functions as BORROWED. Cross-check that every one listed actually has a `retain_if_ptr` call. The existing code already has these calls based on the audit (confirmed by reading `mb_closure_get_capture` at closure.rs:100).

3. **Enable EMIT_REFCOUNT_CALLS**: Set `const EMIT_REFCOUNT_CALLS: bool = true` in `codegen/cranelift/mod.rs` line 15. This activates all conditional refcount emission blocks in both mod.rs (AOT) and jit.rs (JIT).

4. **Re-enable GC**: Set `enabled: true` in `GcState::new()` in `runtime/gc.rs` line 50. With refcounting active, GC only handles cyclic garbage.

5. **Update tests**: The existing `jit_refcount_audit_tests.rs` has tests that verify EMIT_REFCOUNT_CALLS=true and GC enabled. These should pass after the fixes. Run conformance suite to verify no regressions.

### Key constraints

- The closure fix must cascade-release ALL contained MbValues: captures Vec, func, and defaults Vec
- retain_if_ptr is already a no-op for non-pointer values (ints, bools, floats, None) so no risk of over-retaining NaN-boxed scalars
- Immortal objects (rc == u32::MAX) are skipped by both retain and release — compile-time string constants are safe
- Phase ordering: closure fix BEFORE enabling the flag; flag BEFORE enabling GC
- Must pass `cargo test -p mamba --test jit_refcount_audit_tests -- --test-threads=1` without SIGBUS
- Must pass full conformance suite with zero regressions

### Integration points

- `codegen/cranelift/mod.rs` and `codegen/cranelift/jit.rs`: Both import and gate on `EMIT_REFCOUNT_CALLS`
- `runtime/closure.rs`: `mb_closure_release` is called from JIT-compiled code via `runtime_symbols()` registration
- `runtime/gc.rs`: `gc_track`/`gc_untrack` already called by MbObject constructors; enabling just flips the auto-collection trigger
- `runtime/rc.rs`: `retain_if_ptr`/`release_if_ptr` already exist and are used by ~22 borrowed-reference functions
- `tests/jit_refcount_audit_tests.rs`: Existing test file with 20+ tests specifically for this feature
