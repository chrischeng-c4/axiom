"""Hot-loop bench for `httpx.Client` / `httpx.AsyncClient` /
`httpx.Response` / `httpx.Request` module-attribute reads (#1488).

End-user scenario: HTTP client code paths (sync server adapters,
async event-loop dispatchers, request retry wrappers) re-resolve
`httpx.Client` / `httpx.AsyncClient` (constructor), `httpx.Response`
(isinstance / typing), and `httpx.Request` (constructor / typing)
on every request-cycle invocation. Wrapper code that builds a
per-call client or matches on Response/Request types re-resolves
these names through the module's attribute table on each call site.
That per-call module-attribute quad-read is the workload measured
here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 all four are top-level classes registered into
the `httpx` module dict). Mamba's shim returns the same
identity-stable sentinels directly from a dense constant table in
the `httpx` module-attribute resolver, short-circuiting CPython's
module-dict probe chain for read-only sentinels.

Workload: 20_000 paired reads of `Client`, `AsyncClient`, `Response`,
and `Request` per iteration (ITERS scaled to 20_000 so 4 attrs x 20k
= ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import httpx as _hx

_CLIENT_BASELINE = _hx.Client
_ASYNC_CLIENT_BASELINE = _hx.AsyncClient
_RESPONSE_BASELINE = _hx.Response
_REQUEST_BASELINE = _hx.Request

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = _hx.Client
    b = _hx.AsyncClient
    c = _hx.Response
    d = _hx.Request
    if (a is _CLIENT_BASELINE
            and b is _ASYNC_CLIENT_BASELINE
            and c is _RESPONSE_BASELINE
            and d is _REQUEST_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"httpx module-attribute read acc drift: acc={acc} expected={ITERS}"
print("httpx_type_read_hot:", acc)
