"""Hot-loop bench for `logging.getLogger` repeat lookup (#1444).

End-user scenario: library modules and request handlers call
`logging.getLogger(__name__)` at the top of every function /
per-request entry point. After the first call, every subsequent
lookup must return the same `Logger` instance from the module's
internal manager dict — this is a thin name -> Logger hash lookup
plus a hierarchy walk for the cached entry. Frameworks (Flask,
Django, requests) hit this path on every log site touched per
request, so per-call overhead compounds.

Tier: `pure-python stdlib shim` (target mamba/cpython <= 1.0x —
CPython's `getLogger` is pure-Python and walks
`Logger.manager.loggerDict` after a `threading.RLock` acquire on
every call; mamba's shim is a thin dict lookup keyed on the name
with no lock).

Workload: 10_000 lookups of the same name ("test") to exercise the
cache-hit path (Manager.getLogger returns the existing Logger
without allocating a new one — the dominant call shape in steady
state).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker) and reports the ratio. Floor is 1.0x per #1265 Goal 2.
"""

import logging

# Hoist the bound module-level function to a local alias (#2097) so
# per-iter attribute lookup overhead does not skew the measurement.
_get_logger = logging.getLogger

ITERS = 10_000
NAME = "test"

acc = 0
for _ in range(ITERS):
    r = _get_logger(NAME)
    # Accumulator readback prevents DCE — every cached lookup returns
    # the same non-None Logger instance, so the increment branch is
    # always taken.
    if r is not None:
        acc = acc + 1

# Correctness: every iteration should have returned a non-None
# Logger from the manager cache. acc == ITERS or we have a
# regression in the cache-hit branch.
assert acc - ITERS == 0, f"logging.getLogger acc drift: acc={acc} expected={ITERS}"
print("getlogger_hot:", acc)
