"""Hot-loop bench for `orjson.dumps` / `orjson.loads` /
`orjson.JSONDecodeError` / `orjson.JSONEncodeError` module-attribute
reads (#1500).

End-user scenario: orjson-using services re-resolve `orjson.dumps`
(encode entry), `orjson.loads` (decode entry),
`orjson.JSONDecodeError`, and `orjson.JSONEncodeError` on every
call site. Per-call attribute resolution goes through the
`orjson` module's attribute table on each call site. That
per-call module-attribute quadruple-read is the workload measured
here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 the four entries are attached to the `orjson`
module via Python-side wrappers). Mamba's shim returns the same
identity-stable sentinels directly from a dense constant table
in the `orjson` module-attribute resolver, short-circuiting
CPython's module-dict probe chain for read-only sentinels.

Workload: 20_000 paired reads of `dumps`, `loads`,
`JSONDecodeError`, and `JSONEncodeError` per iteration (ITERS
scaled so 4 attrs x 20_000 = ~80k attr-reads per run, matching
the cross-tier 80k attr-read budget used by the 4-attr 3p
perf-pin family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import orjson


_DUMPS_BASELINE = orjson.dumps
_LOADS_BASELINE = orjson.loads
_JDE_BASELINE = orjson.JSONDecodeError
_JEE_BASELINE = orjson.JSONEncodeError

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = orjson.dumps
    b = orjson.loads
    c = orjson.JSONDecodeError
    d = orjson.JSONEncodeError
    if (a is _DUMPS_BASELINE
            and b is _LOADS_BASELINE
            and c is _JDE_BASELINE
            and d is _JEE_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"orjson module-attribute read acc drift: acc={acc} expected={ITERS}"
print("orjson_type_read_hot:", acc)
