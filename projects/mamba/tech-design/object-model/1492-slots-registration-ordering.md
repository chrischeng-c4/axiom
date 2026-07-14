# #1492 — `__slots__` registration must follow runtime base resolution

Status: landed (`dde0a6e98` fix-pack; re-adapted to upstream registration
struct during the 2026-07-14 rebase). Backfill TD.

## Mechanism

`emit_class_registration` emitted `mb_register_slots` inline right after
`mb_class_define_multi`, BEFORE `mb_class_update_bases` ran for classes whose
bases resolve at their textual `ClassDefPlaceholder`. `mb_register_slots`
merges parent slots from the MRO at call time — with a pre-update MRO it saw
no bases and dropped inherited slot names (Child lost Base's `x`).

## Invariant

For any class with runtime-resolved bases: `mb_class_define_*` →
`mb_class_update_bases` → `mb_register_slots`, in that order. Statically-based
classes keep immediate registration (R3).

## Fix pattern

Queue-and-drain: `pending_class_slots: Vec<(runtime_key, SymbolId, Vec<slot>)>`
filled during registration; `emit_class_slots_for(sym)` drains AFTER
`emit_runtime_class_bases_for` at both call sites (placeholder path + fallback
loop). Name vreg via the cached `class_runtime_key_value(sym, fallback)`
helper (upstream's indirection), mirroring `emit_class_attrs_for`.

## Verification contract

Rust locks: `test_runtime_base_slots_register_after_base_update` (MIR order),
`runtime_base_slots_include_inherited_fields_before_instance_init` (e2e).
Probe: Base/Child slotted inheritance with zero-arg super — values identical
vs oracle. Known adjacent divergence NOT covered here: `cls.__slots__`
attribute reports merged layout instead of declared tuple (#1523, open).
