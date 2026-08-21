"""Hot-loop bench for `csv.QUOTE_ALL` quoting-constant read (#1454).

End-user scenario: CSV writers / dialect configurators (pandas
to_csv, csv.writer call sites, ETL scripts) that branch on the
canonical quoting constants on every dialect instantiation or
per-row write. The cost is a single module-attribute lookup
returning a small `int` (1 for `QUOTE_ALL`, 0 for `QUOTE_MINIMAL`,
2 for `QUOTE_NONNUMERIC`, 3 for `QUOTE_NONE`), but it appears in
every dialect-resolution fast path so the per-access constant
factor is what matters. Mamba's edge is collapsing the lookup
chain (module dict -> int ref) into a direct constant-int load.

Tier: `runtime constant` (target mamba/cpython <= 1.0x — CPython's
`csv.QUOTE_ALL` access is a `module.__dict__` lookup; the value is
a plain `int` whose identity is stable across the whole process).

Workload: 10_000 reads of `csv.QUOTE_ALL` compared against the
expected integer 1. The accumulator is incremented on every
matching read, so a misread immediately fails the correctness
assert and a dead-code elimination of the read would leave
`acc != ITERS`.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker) and reports the ratio. Floor is 1.0x per #1265 Goal 2.
"""

import csv as _csv

# Hoist the bound module to a local alias (#2097) so per-iter
# attribute lookup overhead is the *only* thing we measure — the
# LOAD_GLOBAL -> module-dict lookup chain is the hot path under test.
_mod = _csv
EXPECTED = 1  # canonical csv.QUOTE_ALL value; snapshot for correctness compare

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    v = _mod.QUOTE_ALL
    # Accumulator readback prevents DCE — `v` is the quoting
    # constant (a plain int in both CPython and mamba), so the
    # equality always holds and the increment is always taken.
    if v == EXPECTED:
        acc = acc + 1

# Correctness: every iteration must read back csv.QUOTE_ALL == 1.
# acc == ITERS or we have a regression in module-attribute stability.
assert acc - ITERS == 0, f"csv.QUOTE_ALL acc drift: acc={acc} expected={ITERS}"
print("quote_all_const_hot:", acc)
