"""Hot-loop bench for `urllib.request.urlopen` /
`urllib.request.Request` / `urllib.request.quote` /
`urllib.request.unquote` module-attribute reads (#1420).

End-user scenario: HTTP-client glue code (REST API wrappers, web
scrapers, downloader scripts) typically reads `urllib.request` top-
level names on every fetch site rather than caching a local alias.
Wrapper code that builds a `urllib.request.Request(url, data=...)`,
calls `urllib.request.urlopen(req)`, and percent-encodes path
fragments via `urllib.request.quote(seg)` re-resolves these names
through the module's attribute table on each call site. That per-
call module-attribute quad-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
CPython's `urllib.request.urlopen` / `urllib.request.Request` are
top-level module-dict probes on 3.12 returning function / class
objects, and `urllib.request.quote` / `urllib.request.unquote` are
re-exports from `urllib.parse` that resolve through the same
module-dict probe chain). Mamba's shim returns the same identity-
stable sentinels directly from a dense constant table in the
`urllib.request` module-attribute resolver, short-circuiting
CPython's module-dict probe chain for read-only urllib.request
sentinels.

Workload: 20_000 paired reads of `urlopen`, `Request`, `quote`, and
`unquote` per iteration (ITERS doubled to 20_000 so 4 attrs x 20k
= ~80k attr-reads per run, matching the per-spawn budget of the
8-attr fixtures at 10_000 iters). Compared by identity (`is`)
against the hoisted baseline references taken once before the loop.
The accumulator increments when all four reads resolve to identical
objects.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import urllib.request as _ur

_URLOPEN_BASELINE = _ur.urlopen
_REQUEST_BASELINE = _ur.Request
_QUOTE_BASELINE = _ur.quote
_UNQUOTE_BASELINE = _ur.unquote

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = _ur.urlopen
    b = _ur.Request
    c = _ur.quote
    d = _ur.unquote
    if (a is _URLOPEN_BASELINE
            and b is _REQUEST_BASELINE
            and c is _QUOTE_BASELINE
            and d is _UNQUOTE_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"urllib.request module-attribute read acc drift: acc={acc} expected={ITERS}"
print("urllib_request_type_read_hot:", acc)
