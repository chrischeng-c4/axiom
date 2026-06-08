"""Hot-loop bench for `google.api_core.retry` /
`google.api_core.timeout` /
`google.api_core.exceptions` /
`google.api_core.__version__` module-attribute reads (#1509).

End-user scenario: google-api-core-using services re-resolve
`google.api_core.retry` (retry submodule),
`google.api_core.timeout` (timeout submodule),
`google.api_core.exceptions` (exceptions submodule), and
`google.api_core.__version__` (version string sentinel) on every
call site. Per-call attribute resolution goes through the
`google.api_core` module's attribute table on each call site.
That per-call module-attribute quadruple-read is the workload
measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `retry`, `timeout`, `exceptions`,
and `__version__` per iteration (ITERS scaled so 4 attrs x 20_000 =
~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import google.api_core


_R_BASELINE = google.api_core.retry
_T_BASELINE = google.api_core.timeout
_E_BASELINE = google.api_core.exceptions
_VERSION_BASELINE = google.api_core.__version__

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = google.api_core.retry
    b = google.api_core.timeout
    c = google.api_core.exceptions
    d = google.api_core.__version__
    if (a is _R_BASELINE
            and b is _T_BASELINE
            and c is _E_BASELINE
            and d is _VERSION_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"google.api_core module-attribute read acc drift: acc={acc} expected={ITERS}"
print("google_api_core_type_read_hot:", acc)
