# #238 — unbound `Class.__init__(recv, …)` calls: always skip arg 0

Status: landed (`dde0a6e98` fix-pack). Backfill TD.

## Mechanism

The stdlib-signature arg checker skipped the explicit receiver of an unbound
`__init__` call only when the receiver LOOKED like a fresh construction
(`Class(...)` / `object.__new__(Class)`). The standard idiom
`Base.__init__(self, …)` with a plain `self` identifier was not recognized —
every subsequent arg checked one position early (12 spurious mismatches in a
single codeccallbacks fixture).

## Invariant

`__init__` can never be a classmethod/staticmethod: in `ClassName.__init__(recv, …)`
the first argument IS the instance, regardless of the receiver expression's
shape. `attr == "__init__"` alone is sufficient to skip arg 0 — in BOTH
call-resolution paths (import_origins class_sig branch and the Ty::Class
fallback used for builtin exception classes).

## Verification contract

`behavior/std-libs/codeccallbacks/codec_callback_test__test_encode_odd_bytes_replacement.py`
byte-identical; `types::` lib tests. Note: the checker fix exposes RUNTIME
gaps in the same idiom — unbound chain attr loss is #1557 (open), tracked
separately in `exception-construction-contracts.md`.
