"""Hot-loop bench for `marshmallow.Schema` / `marshmallow.fields` /
`marshmallow.validate` / `marshmallow.ValidationError` module-attribute
reads (#1498).

End-user scenario: marshmallow-using services re-resolve
`marshmallow.Schema` (declarative schema base),
`marshmallow.fields` (field-type submodule),
`marshmallow.validate` (validator submodule), and
`marshmallow.ValidationError` (error type) on every call site.
Per-call attribute resolution goes through the `marshmallow`
module's attribute table on each call site. That per-call
module-attribute quadruple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 the four entries are attached to the
`marshmallow` module via Python-side wrappers). Mamba's shim
returns the same identity-stable sentinels directly from a dense
constant table in the `marshmallow` module-attribute resolver,
short-circuiting CPython's module-dict probe chain for read-only
sentinels.

Workload: 20_000 paired reads of `Schema`, `fields`, `validate`,
and `ValidationError` per iteration (ITERS scaled so 4 attrs x
20_000 = ~80k attr-reads per run, matching the cross-tier 80k
attr-read budget used by the 4-attr 3p perf-pin family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import marshmallow


_SCHEMA_BASELINE = marshmallow.Schema
_FIELDS_BASELINE = marshmallow.fields
_VALIDATE_BASELINE = marshmallow.validate
_VALIDATION_ERROR_BASELINE = marshmallow.ValidationError

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = marshmallow.Schema
    b = marshmallow.fields
    c = marshmallow.validate
    d = marshmallow.ValidationError
    if (a is _SCHEMA_BASELINE
            and b is _FIELDS_BASELINE
            and c is _VALIDATE_BASELINE
            and d is _VALIDATION_ERROR_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"marshmallow module-attribute read acc drift: acc={acc} expected={ITERS}"
print("marshmallow_type_read_hot:", acc)
