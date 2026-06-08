"""Hot-loop bench for `requests.get` / `requests.post` /
`requests.request` / `requests.Session` module-attribute reads
(#1487).

End-user scenario: requests-using services re-resolve
`requests.get` (HTTP GET shortcut), `requests.post` (HTTP POST
shortcut), `requests.request` (generic HTTP entry point), and
`requests.Session` (session class) on every call site. Per-call
attribute resolution goes through the `requests` module's
attribute table on each call site. That per-call module-attribute
quadruple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 the four entries are attached to the `requests`
module via Python-side wrappers). Mamba's shim returns the same
identity-stable sentinels directly from a dense constant table in
the `requests` module-attribute resolver, short-circuiting
CPython's module-dict probe chain for read-only sentinels.

Workload: 20_000 paired reads of `get`, `post`, `request`, and
`Session` per iteration (ITERS scaled so 4 attrs x 20_000 = ~80k
attr-reads per run, matching the cross-tier 80k attr-read budget
used by the 4-attr 3p perf-pin family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import requests


_GET_BASELINE = requests.get
_POST_BASELINE = requests.post
_REQUEST_BASELINE = requests.request
_SESSION_BASELINE = requests.Session

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = requests.get
    b = requests.post
    c = requests.request
    d = requests.Session
    if (a is _GET_BASELINE
            and b is _POST_BASELINE
            and c is _REQUEST_BASELINE
            and d is _SESSION_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"requests module-attribute read acc drift: acc={acc} expected={ITERS}"
print("requests_type_read_hot:", acc)
