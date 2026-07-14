# #228/#226 — super() dispatch through type/object + CPython error semantics

Status: landed (`dde0a6e98` fix-pack). Backfill TD covering both issues.

## Mechanism

Two gaps in the super machinery:
1. (#228) `super().__new__` from a custom metaclass never reached the
   runtime's `type.__new__` construction: the super-MRO walk had no arm for
   `__new__` reaching `type`/`object`, so the lookup returned none →
   `'NoneType' object is not callable`.
2. (#226) `super()` error paths didn't match CPython: zero-arg super with no
   enclosing class context, >2 args, bad arg types, duplicate bases,
   `object.__new__` extra args, non-None `__init__` return — wrong or missing
   errors.

## Invariant + red line

`super_builtin_native_method` routes `__new__`-at-type/object through
`make_unbound_method` (→ existing `type_new_unbound`/`object_new_unbound`).
**RED LINE: `__init__` is deliberately EXCLUDED from that arm** — including it
regresses the ubiquitous no-op `super().__init__()` idiom (the
`SUPER_MISSING_INIT_METHOD` fallback must keep binding the implicit receiver).
Verified by probe before landing; do not "complete the symmetry".

## Fix pattern

- class/mod.rs: the narrowed `__new__` match arm; `compute_mro` duplicate-base
  check; `object_new_unbound` two-branch arity errors; `check_init_return_value`;
  `mb_super_no_args_error` / `mb_super_checked` / `mb_super_argcount_error`
  externs (registered in symbols.rs).
- hir_to_mir.rs: zero-arg super with no enclosing class → `mb_super_no_args_error`;
  explicit `super(...)` → checked/argcount routes.
- Error messages use `class_display_name`, never raw runtime keys (`c9cebdbb7`).

## Verification contract

`_regression/core/language/metaclasses/{behavior,surface}.py`,
`_regression/core/{class_system,mro_super}/errors.py` byte-identical;
super-idiom probe (plain chain / terminates-at-object / metaclass
`super().__new__`) identical. Open sibling gaps: #1525 (`super().__call__`,
direct `type.__init__`), #1581 (`super().__class__` identity).
