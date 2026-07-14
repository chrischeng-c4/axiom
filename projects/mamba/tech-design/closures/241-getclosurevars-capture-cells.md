# #241 — introspection must read closure cells directly, not the active-module map

Status: landed (`dde0a6e98` fix-pack). Backfill TD.

## Mechanism

`inspect.getclosurevars` read captured values via the module-scoped
active-cell map (`ScopedSymbolKey{module,symbol}`). Native stdlib dispatchers
are registered with `__module__='inspect'`, and `with_callable_module` pushes
that as the active module for the call's duration — so the lookup searched
scope `{module:'inspect'}` instead of the closure's defining scope and read
back None.

## Invariant

Introspection of a closure's captures must read the closure's OWN
`capture_ids`/`capture_cells` arrays (1:1 with its freevars at creation),
never any "currently active" scope keyed state.

## Fix pattern

`closure.rs::closure_capture_value_for_id(handle, id)` (direct array read);
`inspect_mod.rs::d_getclosurevars` uses it first, `mb_global_get_id_raw` only
as defensive fallback.

## Verification contract

`behavior/std-libs/inspect/getclosurevars_reports_nonlocals_and_builtins.py`
byte-identical; `runtime::closure::` lib tests. Hazard note for future
introspection work: any native dispatcher runs with ITS module active — never
resolve user-scope names through active-module state inside a dispatcher.
