"""Hot-loop bench for `tomllib.load` / `tomllib.loads` /
`tomllib.TOMLDecodeError` module-attribute reads (#1261).

End-user scenario: tomllib-using config-loading code re-resolves
`load` (file-fp parser), `loads` (string parser), and
`TOMLDecodeError` (exception class) on every call site. Per-call
attribute resolution goes through the `tomllib` module's attribute
table on each call site. That per-call module-attribute triple-read
is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `load`, `loads`, and
`TOMLDecodeError` per iteration (ITERS scaled so 3 attrs
x 20_000 = ~60k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import tomllib


_L_BASELINE = tomllib.load
_LS_BASELINE = tomllib.loads
_E_BASELINE = tomllib.TOMLDecodeError

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = tomllib.load
    b = tomllib.loads
    c = tomllib.TOMLDecodeError
    if (a is _L_BASELINE
            and b is _LS_BASELINE
            and c is _E_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"tomllib module-attribute read acc drift: acc={acc} expected={ITERS}"
print("tomllib_type_read_hot:", acc)
