"""Hot-loop bench for `fastapi.FastAPI` / `fastapi.APIRouter` /
`fastapi.Depends` / `fastapi.HTTPException` module-attribute reads
(#1519).

End-user scenario: ASGI application bootstrap code paths re-resolve
`fastapi.FastAPI` (the top-level app class), `fastapi.APIRouter`
(router shells), `fastapi.Depends` (dependency-injection marker),
and `fastapi.HTTPException` (error-path raiser) on every route
registration / request scope. Per-request handler dispatch
re-resolves these names through the module's attribute table on
each call site. That per-call module-attribute quadruple-read is
the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 `fastapi.FastAPI`, `fastapi.APIRouter`,
`fastapi.Depends`, and `fastapi.HTTPException` are top-level
classes / functions routed through the `fastapi` module dict).
Mamba's shim returns the same identity-stable sentinels directly
from a dense constant table in the `fastapi` module-attribute
resolver, short-circuiting CPython's module-dict probe chain for
read-only sentinels.

Workload: 20_000 paired reads of `FastAPI`, `APIRouter`, `Depends`,
and `HTTPException` per iteration (ITERS scaled so 4 attrs x 20_000
= ~80k attr-reads per run, matching the cross-tier 80k attr-read
budget used by the 4-attr 3p perf-pin family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import fastapi as _fa

_FASTAPI_BASELINE = _fa.FastAPI
_APIROUTER_BASELINE = _fa.APIRouter
_DEPENDS_BASELINE = _fa.Depends
_HTTPEXCEPTION_BASELINE = _fa.HTTPException

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = _fa.FastAPI
    b = _fa.APIRouter
    c = _fa.Depends
    d = _fa.HTTPException
    if (a is _FASTAPI_BASELINE
            and b is _APIROUTER_BASELINE
            and c is _DEPENDS_BASELINE
            and d is _HTTPEXCEPTION_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"fastapi module-attribute read acc drift: acc={acc} expected={ITERS}"
print("fastapi_type_read_hot:", acc)
