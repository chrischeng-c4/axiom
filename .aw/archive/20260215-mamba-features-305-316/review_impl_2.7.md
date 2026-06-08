---
verdict: APPROVED
file: implementation
iteration: 1
task_id: 2.7
---

# Review: implementation:task_2.7 (Iteration 1)

**Change ID**: mamba-features-305-316

## Summary

Task 2.7 implementation matches the mamba-iteration-protocol spec and requested validation points. Verified in source that for-loop lowering uses `mb_next` then `mb_has_next` to distinguish yielded `None` from exhaustion for runtime iterators that preserve exhausted state, callable dispatch is gated by `CALLABLE_REGISTRY` before transmute, StopIteration signaling uses thread-local flag with clear/check around `__next__`, and TypeError diagnostics exist for both instance and primitive non-iterables. Targeted tests passed: `test_iterator_protocol`, `test_range_iterator`, and `runtime::iter::tests::test_instance_without_iter_not_iterable`.

## Checklist

- ✅ R1: Obtain iterator via __iter__
  - `mb_iter` falls back to class `__iter__` lookup/invocation for instances (`iter.rs`).
- ✅ R2: Advance via __next__ until exhaustion
  - User-defined iterators call `__next__` and use `STOP_ITERATION` flag + exhausted state (`iter.rs`).
- ✅ R3: Built-in iterators (list/dict/tuple)
  - Built-in iter kinds include List, DictKeys, Tuple and are advanced in `advance_iter` (`iter.rs`).
- ✅ For-loop lowering order
  - `lower_for` emits `mb_next` then `mb_has_next` (`hir_to_mir.rs`).
- ✅ Callable safety for dunder dispatch
  - `mb_call_method1` validates address in `CALLABLE_REGISTRY` before function-pointer transmute (`class.rs`).
- ✅ TypeError diagnostics for non-iterables
  - Instance-without-`__iter__` and primitive-object paths both emit TypeError diagnostics (`iter.rs`).
- ✅ Task-relevant tests
  - Targeted iterator tests executed and passed under `cargo test -p mamba`.

## Issues

No issues found.

## Verdict

- [x] APPROVED
- [ ] REVIEWED
- [ ] REJECTED

