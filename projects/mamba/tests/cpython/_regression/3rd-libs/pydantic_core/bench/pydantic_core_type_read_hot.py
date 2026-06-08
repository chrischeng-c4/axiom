"""Hot-loop bench for `pydantic_core.ValidationError` /
`pydantic_core.SchemaValidator` / `pydantic_core.SchemaSerializer` /
`pydantic_core.Url` module-attribute reads (#1496).

End-user scenario: pydantic v2 model code paths re-resolve
`pydantic_core.ValidationError` (raise path), `pydantic_core.SchemaValidator`
(validation entry point), `pydantic_core.SchemaSerializer` (serialization
entry point), and `pydantic_core.Url` (URL-typed fields) on every
`BaseModel.model_validate` / `.model_dump` / `.model_dump_json` call.
Wrapper code that builds a SchemaValidator per request hot-path re-resolves
these names through the module's attribute table on each instantiation.
That per-call module-attribute quad-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 these are Rust-backed pyo3 classes registered into the
`pydantic_core` module dict). Mamba's shim returns the same
identity-stable sentinels directly from a dense constant table in the
`pydantic_core` module-attribute resolver, short-circuiting CPython's
module-dict probe chain for read-only sentinels.

Workload: 20_000 paired reads of `ValidationError`, `SchemaValidator`,
`SchemaSerializer`, and `Url` per iteration (ITERS scaled to 20_000 so
4 attrs x 20k = ~80k attr-reads per run, matching the per-spawn budget
of the 8-attr fixtures at 10_000 iters, the 2-attr fixtures at 40_000
iters, and the 1-attr fixtures at 80_000 iters). All four values are
re-resolved from the `pydantic_core` module-attribute table on every
iter (not hoisted to a local) and identity-compared against the hoisted
baseline references; the accumulator increments when all four reads
resolve to identical objects.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import pydantic_core as _pc

_VALIDATION_ERROR_BASELINE = _pc.ValidationError
_SCHEMA_VALIDATOR_BASELINE = _pc.SchemaValidator
_SCHEMA_SERIALIZER_BASELINE = _pc.SchemaSerializer
_URL_BASELINE = _pc.Url

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = _pc.ValidationError
    b = _pc.SchemaValidator
    c = _pc.SchemaSerializer
    d = _pc.Url
    if (a is _VALIDATION_ERROR_BASELINE
            and b is _SCHEMA_VALIDATOR_BASELINE
            and c is _SCHEMA_SERIALIZER_BASELINE
            and d is _URL_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"pydantic_core module-attribute read acc drift: acc={acc} expected={ITERS}"
print("pydantic_core_type_read_hot:", acc)
