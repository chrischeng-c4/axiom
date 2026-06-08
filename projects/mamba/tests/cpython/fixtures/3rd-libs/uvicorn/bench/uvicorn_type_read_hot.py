"""Hot-loop bench for `uvicorn.Config` / `uvicorn.Server` /
`uvicorn.main` / `uvicorn.run` module-attribute reads (#1521).

End-user scenario: ASGI server bootstrap code paths re-resolve
`uvicorn.Config` (the server-config class), `uvicorn.Server`
(the asyncio-driven server class), `uvicorn.main` (the CLI entry
point), and `uvicorn.run` (the high-level run helper) on every
process-launch / multi-worker fan-out / programmatic boot. That
per-call module-attribute quadruple-read is the workload measured
here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 `uvicorn.Config`, `uvicorn.Server`, `uvicorn.main`,
and `uvicorn.run` are top-level classes / functions routed through
the `uvicorn` module dict, and form the package's own `__all__`).
Mamba's shim returns the same identity-stable sentinels directly
from a dense constant table in the `uvicorn` module-attribute
resolver, short-circuiting CPython's module-dict probe chain for
read-only sentinels.

Workload: 20_000 paired reads of `Config`, `Server`, `main`, and
`run` per iteration (ITERS scaled so 4 attrs x 20_000 = ~80k
attr-reads per run, matching the cross-tier 80k attr-read budget
used by the 4-attr 3p perf-pin family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import uvicorn as _uv

_CONFIG_BASELINE = _uv.Config
_SERVER_BASELINE = _uv.Server
_MAIN_BASELINE = _uv.main
_RUN_BASELINE = _uv.run

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = _uv.Config
    b = _uv.Server
    c = _uv.main
    d = _uv.run
    if (a is _CONFIG_BASELINE
            and b is _SERVER_BASELINE
            and c is _MAIN_BASELINE
            and d is _RUN_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"uvicorn module-attribute read acc drift: acc={acc} expected={ITERS}"
print("uvicorn_type_read_hot:", acc)
