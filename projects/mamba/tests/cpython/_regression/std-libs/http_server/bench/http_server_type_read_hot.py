"""Hot-loop bench for `http.server.HTTPServer` /
`http.server.BaseHTTPRequestHandler` /
`http.server.SimpleHTTPRequestHandler` /
`http.server.CGIHTTPRequestHandler` /
`http.server.ThreadingHTTPServer` module-attribute reads (#1418).

End-user scenario: dev-server / fixture / static-asset launcher
code (test rigs, CI smoke harnesses, local-dev replacements for a
full WSGI stack) typically reads the `http.server` module-level
class names on every spawn site rather than caching a local alias.
Wrapper code that branches on
`isinstance(handler, http.server.SimpleHTTPRequestHandler)` or
constructs a one-shot listener via
`http.server.ThreadingHTTPServer(addr, handler)` re-resolves these
names through the module's attribute table on each call site. That
per-call module-attribute quint-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
CPython's `http.server.HTTPServer` family are top-level module-dict
probes on 3.12 returning class objects). Mamba's shim returns the
same identity-stable sentinels directly from a dense constant table
in the `http.server` module-attribute resolver, short-circuiting
CPython's module-dict probe chain for read-only class sentinels.

Workload: 10_000 paired reads of `http.server.HTTPServer`,
`http.server.BaseHTTPRequestHandler`,
`http.server.SimpleHTTPRequestHandler`,
`http.server.CGIHTTPRequestHandler`, and
`http.server.ThreadingHTTPServer` per iteration, compared by
identity (`is`) against the hoisted baseline references taken once
before the loop. The accumulator increments when all five reads
resolve to identical objects; a misread (different identity / wrong
binding) would immediately fail the correctness assert and dead-code
elimination of any read would leave `acc != ITERS`.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import http.server as _http_server

_HTTPSERVER_BASELINE = _http_server.HTTPServer
_BASEHANDLER_BASELINE = _http_server.BaseHTTPRequestHandler
_SIMPLEHANDLER_BASELINE = _http_server.SimpleHTTPRequestHandler
_CGIHANDLER_BASELINE = _http_server.CGIHTTPRequestHandler
_THREADINGSERVER_BASELINE = _http_server.ThreadingHTTPServer

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    a = _http_server.HTTPServer
    b = _http_server.BaseHTTPRequestHandler
    c = _http_server.SimpleHTTPRequestHandler
    d = _http_server.CGIHTTPRequestHandler
    e = _http_server.ThreadingHTTPServer
    if (a is _HTTPSERVER_BASELINE
            and b is _BASEHANDLER_BASELINE
            and c is _SIMPLEHANDLER_BASELINE
            and d is _CGIHANDLER_BASELINE
            and e is _THREADINGSERVER_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"http.server module-attribute read acc drift: acc={acc} expected={ITERS}"
print("http_server_type_read_hot:", acc)
