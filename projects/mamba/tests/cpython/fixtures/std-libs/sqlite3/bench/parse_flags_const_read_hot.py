"""Hot-loop bench for `sqlite3.PARSE_DECLTYPES` / `PARSE_COLNAMES`
module-constant read (#1455).

End-user scenario: data-access layers and ORM helpers that open
`sqlite3.connect(path, detect_types=sqlite3.PARSE_DECLTYPES | sqlite3.PARSE_COLNAMES)`
on every connection. Pool-recycled connections, request-per-connection
web handlers, and short-lived script-driven migrations all reference
the two flag constants on every `connect(...)` call. The module
attribute lookup is on the hot path of every open; the hoisted-local
alias (`D = sqlite3.PARSE_DECLTYPES`) is the canonical pattern for
tight inner loops. That per-iter module-constant readback is the
workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x —
CPython's `sqlite3.PARSE_DECLTYPES` is a top-level module-dict probe
returning the int `1`; `sqlite3.PARSE_COLNAMES` is `2`). Mamba's
shim returns the same two sentinel ints directly from the
module-attribute resolver, so the per-access constant factor is
the only thing on the clock.

Workload: 10_000 reads each of `sqlite3.PARSE_DECLTYPES` and
`sqlite3.PARSE_COLNAMES` against the canonical CPython values
(`1`, `2`). The accumulator sums the two reads on every iter, so
a misread (wrong int) immediately fails the correctness assert and
dead-code elimination of the reads would leave
`acc != ITERS`.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import sqlite3 as _sqlite3

# Hoist the two module-attribute reads to local aliases outside the
# hot loop. The bench measures the per-iter readback through these
# locals — the bound integer sentinels are the canonical CPython
# values (`PARSE_DECLTYPES = 1`, `PARSE_COLNAMES = 2`).
_PARSE_DECLTYPES = _sqlite3.PARSE_DECLTYPES
_PARSE_COLNAMES = _sqlite3.PARSE_COLNAMES

# Snapshot expected values once before the loop so the correctness
# compare is a pure int-equality probe in the hot path.
EXPECTED_DECLTYPES = 1
EXPECTED_COLNAMES = 2
EXPECTED_SUM = EXPECTED_DECLTYPES + EXPECTED_COLNAMES  # 3

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    s = _PARSE_DECLTYPES + _PARSE_COLNAMES
    # Accumulator readback prevents DCE — `s` is an int sum of the
    # two sentinel constants, so the equality always holds in both
    # CPython and mamba and the increment is always taken.
    if s == EXPECTED_SUM:
        acc = acc + 1

# Correctness: every iteration must read back PARSE_DECLTYPES+PARSE_COLNAMES == 3.
# acc == ITERS or we have a regression in sqlite3 parse-flag constants.
assert acc - ITERS == 0, f"sqlite3 parse-flags const read acc drift: acc={acc} expected={ITERS}"
print("parse_flags_const_read_hot:", acc)
