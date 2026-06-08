"""Hot-loop bench for `jsonschema.validate` /
`jsonschema.Draft7Validator` / `jsonschema.SchemaError` /
`jsonschema.ValidationError` module-attribute reads (#1497).

End-user scenario: jsonschema-using services re-resolve
`jsonschema.validate` (one-shot validation entry),
`jsonschema.Draft7Validator` (validator class),
`jsonschema.SchemaError` (schema-level error type), and
`jsonschema.ValidationError` (error type) on every call site.
Per-call attribute resolution goes through the `jsonschema`
module's attribute table on each call site. That per-call
module-attribute quadruple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 the four entries are attached to the
`jsonschema` module via Python-side wrappers). Mamba's shim
returns the same identity-stable sentinels directly from a dense
constant table in the `jsonschema` module-attribute resolver,
short-circuiting CPython's module-dict probe chain for read-only
sentinels.

Workload: 20_000 paired reads of `validate`, `Draft7Validator`,
`SchemaError`, and `ValidationError` per iteration (ITERS scaled
so 4 attrs x 20_000 = ~80k attr-reads per run, matching the
cross-tier 80k attr-read budget used by the 4-attr 3p perf-pin
family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import jsonschema


_VALIDATE_BASELINE = jsonschema.validate
_DRAFT7_BASELINE = jsonschema.Draft7Validator
_SCHEMA_ERROR_BASELINE = jsonschema.SchemaError
_VALIDATION_ERROR_BASELINE = jsonschema.ValidationError

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = jsonschema.validate
    b = jsonschema.Draft7Validator
    c = jsonschema.SchemaError
    d = jsonschema.ValidationError
    if (a is _VALIDATE_BASELINE
            and b is _DRAFT7_BASELINE
            and c is _SCHEMA_ERROR_BASELINE
            and d is _VALIDATION_ERROR_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"jsonschema module-attribute read acc drift: acc={acc} expected={ITERS}"
print("jsonschema_type_read_hot:", acc)
