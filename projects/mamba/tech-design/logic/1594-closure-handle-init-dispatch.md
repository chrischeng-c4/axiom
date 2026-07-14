# #1594 — closure-wrapped `__init__` must unwrap before dispatch caching

Status: landed `7a23d99dc` (2026-07-13). Backfill TD.

## Mechanism

#1379's strict-type work made class-cell capture unconditional: any method
using bare `__class__` OR zero-arg `super()` becomes a closure (int-tagged
handle) instead of a plain function. `extract_func_addr` does not understand
closure handles → returns a garbage pseudo-address never present in
`CALLABLE_REGISTRY` → `call_init_for_instance`'s cached fast path recorded
`is_registered=false` → construction silently skipped the whole `__init__`
(no body, no MRO init, no exception).

## Invariant

Any code path that computes/caches a dispatch address for a method value must
unwrap closure handles first: use `extract_registered_func_addr` (which is how
`CALLABLE_REGISTRY` itself is populated), never `extract_func_addr`, when the
value may be a method.

## Fix pattern

`runtime/class/mod.rs`, 5 call sites (~1590/1963/2476/3522/3548):
`extract_func_addr(init_method)` → `extract_registered_func_addr(init_method)`.

## Verification contract

Discrimination matrix probes (scratchpad probe-trigger.py shapes): zero-arg
`super()` in `__init__` (chaining and non-chaining), `super().m()` in
`__init__`, unbound `A.__init__(self)` — all byte-identical vs python3.12.
Regression locks: `lower::hir_to_mir::tests::test_runtime_base_slots_register_after_base_update`,
`driver::tests::runtime_base_slots_include_inherited_fields_before_instance_init`.
Corpus impact was massive (this fix + #1610 dominate the 3,878→1,483 delta).
