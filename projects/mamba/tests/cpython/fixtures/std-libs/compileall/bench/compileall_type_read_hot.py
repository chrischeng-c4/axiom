"""Hot-loop bench for `compileall.compile_dir` /
`compileall.compile_file` / `compileall.compile_path` /
`compileall.main` module-attribute reads (#1261).

End-user scenario: compileall-using build / packaging code
re-resolves `compile_dir` (recursive .py -> .pyc), `compile_file`
(single .py -> .pyc), `compile_path` (sys.path walk), and `main`
(CLI entry) on every call site. Per-call attribute resolution
goes through the `compileall` module's attribute table on each
call site. That per-call module-attribute quadruple-read is the
workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `compile_dir`, `compile_file`,
`compile_path`, and `main` per iteration (ITERS scaled so 4 attrs
x 20_000 = ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import compileall

import sys

_CD_BASELINE = compileall.compile_dir
_CF_BASELINE = compileall.compile_file
_CP_BASELINE = compileall.compile_path
_M_BASELINE = compileall.main

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = compileall.compile_dir
    b = compileall.compile_file
    c = compileall.compile_path
    d = compileall.main
    if (a is _CD_BASELINE
            and b is _CF_BASELINE
            and c is _CP_BASELINE
            and d is _M_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"compileall module-attribute read acc drift: acc={acc} expected={ITERS}"
print("compileall_type_read_hot:", acc)
