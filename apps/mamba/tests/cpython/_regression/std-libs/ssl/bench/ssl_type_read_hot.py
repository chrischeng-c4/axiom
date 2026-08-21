"""Hot-loop bench for `ssl.SSLContext` / `ssl.create_default_context`
/ `ssl.SSLError` / `ssl.PROTOCOL_TLS` module-attribute reads (#1414).

End-user scenario: TLS-using services (requests, httpx, aiohttp, urllib3,
boto3, psycopg, asyncpg) re-resolve `ssl.SSLContext` (TLS context
factory), `ssl.create_default_context` (recommended factory helper),
`ssl.SSLError` (catch-block exception class), and `ssl.PROTOCOL_TLS`
(protocol selector constant) on every TLS handshake call site. Per-
request TLS dispatch re-resolves these names through the `ssl` module's
attribute table on each call site. That per-call module-attribute
quadruple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 the four entries are attached to the `ssl` module
via the C-extension `_ssl` import + Python-side wrappers).
Mamba's shim returns the same identity-stable sentinels directly
from a dense constant table in the `ssl` module-attribute resolver,
short-circuiting CPython's module-dict probe chain for read-only
sentinels.

Workload: 20_000 paired reads of `SSLContext`, `create_default_context`,
`SSLError`, and `PROTOCOL_TLS` per iteration (ITERS scaled so
4 attrs x 20_000 = ~80k attr-reads per run, matching the
cross-tier 80k attr-read budget used by the 4-attr 3p perf-pin
family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import ssl


_SSL_CONTEXT_BASELINE = ssl.SSLContext
_CREATE_DEFAULT_CONTEXT_BASELINE = ssl.create_default_context
_SSL_ERROR_BASELINE = ssl.SSLError
_PROTOCOL_TLS_BASELINE = ssl.PROTOCOL_TLS

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = ssl.SSLContext
    b = ssl.create_default_context
    c = ssl.SSLError
    d = ssl.PROTOCOL_TLS
    if (a is _SSL_CONTEXT_BASELINE
            and b is _CREATE_DEFAULT_CONTEXT_BASELINE
            and c is _SSL_ERROR_BASELINE
            and d is _PROTOCOL_TLS_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"ssl module-attribute read acc drift: acc={acc} expected={ITERS}"
print("ssl_type_read_hot:", acc)
