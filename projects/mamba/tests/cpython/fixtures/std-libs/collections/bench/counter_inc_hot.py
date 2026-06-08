"""Hot-loop bench for `collections.Counter` per-key increment (#1449).

End-user scenario: a real-time histogram / frequency table — log
ingestion, token frequency, request-path counter — increments per-key
counts millions of times per second. The interesting cost is the
`__getitem__` (default-0 lookup) + `__setitem__` round trip per
increment, where mamba's dispatcher overhead competes against CPython's
C-level `dict.__contains__` + branchful update path.

Tier: `heap container-light` (target mamba/cpython <= 1.0x — CPython's
Counter is a Python subclass of dict whose `__missing__` returns 0;
mamba's edge is a thin Rust dispatcher over the underlying dict that
folds the missing-key branch into a single lookup-or-zero primitive).

Workload: 10_000 increments across a small key space (5 distinct keys)
to keep the working set in L1 and isolate per-call overhead.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
and reports the ratio. Floor is 1.0x per #1265 Goal 2.
"""

import collections

Counter = collections.Counter

ITERS = 10_000
# Small key space — 5 distinct keys, cycled.
KEYS = ("alpha", "beta", "gamma", "delta", "epsilon")

c = Counter()
for i in range(ITERS):
    k = KEYS[i % 5]
    c[k] = c[k] + 1

# Each of 5 keys should land 10_000/5 = 2000 hits.
total = 0
for k in KEYS:
    total = total + c[k]
expected = ITERS
assert total - expected == 0, f"counter inc drift: total={total} expected={expected}"
print("counter_inc_hot:", total)
