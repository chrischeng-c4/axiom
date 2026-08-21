"""Hot-loop bench for `pydantic.BaseModel` / `pydantic.Field` /
`pydantic.ValidationError` / `pydantic.TypeAdapter` module-attribute
reads (#1495).

End-user scenario: pydantic v2 model definitions and runtime adapters
re-resolve `pydantic.BaseModel` (subclass base), `pydantic.Field`
(per-field constraint declaration), `pydantic.ValidationError`
(exception catch), and `pydantic.TypeAdapter` (one-shot validation
entry point) on every module-level definition and every per-request
validate/serialize call. Wrapper code that builds a per-payload
TypeAdapter or catches ValidationError re-resolves these names
through the module's attribute table on each call site. That
per-call module-attribute quad-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 `pydantic.BaseModel` is a metaclass-built class,
`pydantic.Field` and `pydantic.TypeAdapter` are factory callables,
and `pydantic.ValidationError` is a re-export from `pydantic_core`,
all routed through the `pydantic` module dict). Mamba's shim
returns the same identity-stable sentinels directly from a dense
constant table in the `pydantic` module-attribute resolver,
short-circuiting CPython's module-dict probe chain for read-only
sentinels.

Workload: 20_000 paired reads of `BaseModel`, `Field`,
`ValidationError`, and `TypeAdapter` per iteration (ITERS scaled to
20_000 so 4 attrs x 20k = ~80k attr-reads per run, matching the
per-spawn budget of the 8-attr fixtures at 10_000 iters, the 2-attr
fixtures at 40_000 iters, and the 1-attr fixtures at 80_000 iters).
All four values are re-resolved from the `pydantic` module-attribute
table on every iter (not hoisted to a local) and identity-compared
against the hoisted baseline references; the accumulator increments
when all four reads resolve to identical objects.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import pydantic as _pyd

_BASE_MODEL_BASELINE = _pyd.BaseModel
_FIELD_BASELINE = _pyd.Field
_VALIDATION_ERROR_BASELINE = _pyd.ValidationError
_TYPE_ADAPTER_BASELINE = _pyd.TypeAdapter

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = _pyd.BaseModel
    b = _pyd.Field
    c = _pyd.ValidationError
    d = _pyd.TypeAdapter
    if (a is _BASE_MODEL_BASELINE
            and b is _FIELD_BASELINE
            and c is _VALIDATION_ERROR_BASELINE
            and d is _TYPE_ADAPTER_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"pydantic module-attribute read acc drift: acc={acc} expected={ITERS}"
print("pydantic_type_read_hot:", acc)
