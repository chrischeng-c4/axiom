"""Hot-loop bench for `http.client.OK` status-constant read (#1417).

End-user scenario: HTTP server / client frameworks (WSGI handlers,
FastAPI middleware, requests-style response checkers) that branch on
canonical status-code constants on every response. The cost is a
single module-attribute lookup returning a small `int` (200 for `OK`,
404 for `NOT_FOUND`, 500 for `INTERNAL_SERVER_ERROR`), but it appears
in every per-response fast path so the per-access constant factor is
what matters. Mamba's edge is collapsing the lookup chain (module
dict -> int ref) into a direct constant-int load.

Tier: `runtime constant` (target mamba/cpython <= 1.0x — CPython's
`http.client.OK` access is a `module.__dict__` lookup; the value is a
real `HTTPStatus` IntEnum member whose identity is stable across the
whole process and which equals the plain int 200).

Workload: 10_000 reads of `http.client.OK` compared against the
expected integer 200. The accumulator is incremented on every
matching read, so a misread immediately fails the correctness assert
and a dead-code elimination of the read would leave `acc != ITERS`.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
and reports the ratio. Floor is 1.0x per #1265 Goal 2.
"""

import http.client as _hc

# Hoist the bound module to a local alias (#2097) so per-iter
# attribute lookup overhead is the *only* thing we measure — the
# LOAD_GLOBAL -> module-dict lookup chain is the hot path under test.
_mod = _hc
EXPECTED = 200  # canonical HTTP 200 OK; snapshot for correctness compare

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    v = _mod.OK
    # Accumulator readback prevents DCE — `v` is the status constant
    # (an IntEnum member in CPython, a plain int in mamba), so the
    # equality always holds and the increment is always taken.
    if v == EXPECTED:
        acc = acc + 1

# Correctness: every iteration must read back HTTP 200. acc == ITERS
# or we have a regression in module-attribute stability.
assert acc - ITERS == 0, f"http.client.OK acc drift: acc={acc} expected={ITERS}"
print("status_const_hot:", acc)
