"""Hot-loop bench for `attrs.define` / `attrs.field` / `attrs.asdict` /
`attrs.fields` module-attribute reads (#1493).

End-user scenario: attrs-using services re-resolve `attrs.define`
(primary class decorator), `attrs.field` (descriptor factory),
`attrs.asdict` (instance-to-dict serializer), and `attrs.fields`
(class-introspection helper) on every class declaration and every
serialize/introspect site. Per-call attribute resolution goes through
the `attrs` module's attribute table on each call site. That per-call
module-attribute quadruple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 the four entries are attached to the `attrs` module
via Python-side wrappers).
Mamba's shim returns the same identity-stable sentinels directly
from a dense constant table in the `attrs` module-attribute resolver,
short-circuiting CPython's module-dict probe chain for read-only
sentinels.

Workload: 20_000 paired reads of `define`, `field`, `asdict`, and
`fields` per iteration (ITERS scaled so 4 attrs x 20_000 = ~80k
attr-reads per run, matching the cross-tier 80k attr-read budget used
by the 4-attr 3p perf-pin family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import attrs


_DEFINE_BASELINE = attrs.define
_FIELD_BASELINE = attrs.field
_ASDICT_BASELINE = attrs.asdict
_FIELDS_BASELINE = attrs.fields

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = attrs.define
    b = attrs.field
    c = attrs.asdict
    d = attrs.fields
    if (a is _DEFINE_BASELINE
            and b is _FIELD_BASELINE
            and c is _ASDICT_BASELINE
            and d is _FIELDS_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"attrs module-attribute read acc drift: acc={acc} expected={ITERS}"
print("attrs_type_read_hot:", acc)
