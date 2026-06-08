"""Hot-loop bench for `os.getpid` top-level operation (#1431).

End-user scenario: process-aware logging / tracing libraries that
embed the current PID in every log record header (structured-log
emitters, distributed-trace span tags, per-request audit lines).
The hot path is a single `os.getpid()` call per record, and the
interesting cost is the cross-language boundary into the libc
`getpid(2)` syscall plus the int boxing on return.

Tier: `os shim` (target mamba/cpython <= 1.0x — CPython's
`os.getpid` is a thin C wrapper around `getpid(2)`, so mamba has
to compete with a near-native baseline; the edge comes from
mamba's Rust-side int return path avoiding CPython's PyLong
allocator round trip for small ints).

Workload: 10_000 calls to `os.getpid()` against a stable PID
(the running interpreter's own). Hoist the bound module-level
function to a local alias (#2097) so per-iter attribute lookup
overhead does not skew the measurement. Accumulator readback
prevents DCE — every PID is positive, so the `> 0` branch is
always taken.

DoD: exits 0 under both CPython and mamba; the cross-runtime
bench harness compares per-iteration internal time
per #1265 Goal 2.
"""

import os

# Hoist the bound module-level function to a local alias (#2097) so
# per-iter attribute lookup overhead does not skew the measurement.
_getpid = os.getpid

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    r = _getpid()
    # Accumulator readback prevents DCE — a live PID is always
    # positive on POSIX, so the increment branch is always taken.
    if r > 0:
        acc = acc + 1

# Correctness: every iteration should have observed a positive PID.
# acc == ITERS or we have a regression in the success-path branch.
assert acc - ITERS == 0, f"os.getpid acc drift: acc={acc} expected={ITERS}"
print("getpid_hot:", acc)
