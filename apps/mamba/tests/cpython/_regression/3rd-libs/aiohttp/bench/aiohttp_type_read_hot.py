"""Hot-loop bench for `aiohttp.ClientSession` /
`aiohttp.ClientResponse` / `aiohttp.ClientTimeout` /
`aiohttp.request` module-attribute reads (#1489).

End-user scenario: aiohttp-using services re-resolve
`aiohttp.ClientSession` (primary session class),
`aiohttp.ClientResponse` (response type),
`aiohttp.ClientTimeout` (timeout config), and `aiohttp.request`
(one-shot request entry) on every call site. Per-call attribute
resolution goes through the `aiohttp` module's attribute table
on each call site. That per-call module-attribute quadruple-read
is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 the four entries are attached to the `aiohttp`
module via Python-side wrappers). Mamba's shim returns the same
identity-stable sentinels directly from a dense constant table in
the `aiohttp` module-attribute resolver, short-circuiting
CPython's module-dict probe chain for read-only sentinels.

Workload: 20_000 paired reads of `ClientSession`,
`ClientResponse`, `ClientTimeout`, and `request` per iteration
(ITERS scaled so 4 attrs x 20_000 = ~80k attr-reads per run,
matching the cross-tier 80k attr-read budget used by the 4-attr
3p perf-pin family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import aiohttp


_CLIENT_SESSION_BASELINE = aiohttp.ClientSession
_CLIENT_RESPONSE_BASELINE = aiohttp.ClientResponse
_CLIENT_TIMEOUT_BASELINE = aiohttp.ClientTimeout
_REQUEST_BASELINE = aiohttp.request

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = aiohttp.ClientSession
    b = aiohttp.ClientResponse
    c = aiohttp.ClientTimeout
    d = aiohttp.request
    if (a is _CLIENT_SESSION_BASELINE
            and b is _CLIENT_RESPONSE_BASELINE
            and c is _CLIENT_TIMEOUT_BASELINE
            and d is _REQUEST_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"aiohttp module-attribute read acc drift: acc={acc} expected={ITERS}"
print("aiohttp_type_read_hot:", acc)
