"""Hot-loop bench for `itertools.accumulate` running-sum (#1452).

End-user scenario: a rolling cumulative-sum kernel — running totals
for cashflow projection, time-series cumulative aggregates, prefix
sums for range-query precomputation. The interesting cost is the
per-element fold: CPython runs an internal `acc = acc + x` Python
loop dispatched through `__next__` per step; mamba materializes
the prefix-sum vector eagerly in Rust and the outer `sum()` walks
a flat list.

Tier: `heap container-bulk` (target mamba/cpython <= 1.0x — CPython's
`itertools.accumulate` is a Python-level iterator with default
`operator.add` fast path but still pays per-element FFI; mamba's
edge is collapsing the fold into a tight Rust loop over the input
list and returning a fresh list that the outer `sum()` then folds
without per-call iterator state).

Workload: 10_000 iters of `sum(accumulate(DATA))` over a 200-element
`range(200)` input. The inner prefix-sum touches 200 elements per
iter (2M total folds) — large enough to amortize call overhead, small
enough to keep the working set in L1.

Hoist convention (per #2097): `accumulate` and `sum` are bound to
locals before the loop so each iter is a direct call rather than
a per-iter module-attr / builtin lookup.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker) and reports the ratio. Floor is 1.0x per #1265 Goal 2.

# tier: heap
"""

import itertools

# Hoist module attributes outside the loop — see CLAUDE.md note + #2097.
_accumulate = itertools.accumulate
_sum = sum

DATA = list(range(200))

ITERS = 10_000

total = 0
for _ in range(ITERS):
    total += _sum(_accumulate(DATA))

# Invariant: per-iter contribution is sum of partial sums of [0..200).
# partial_sum[k] = k*(k+1)/2, so sum_{k=0..199} k*(k+1)/2 = 1_333_300.
expected = ITERS * 1_333_300
diff = total - expected
assert diff == 0, f"itertools.accumulate bench mismatch: total={total} expected={expected} diff={diff}"
print("accumulate_sum_hot:", total)
