"""functools.reduce(operator.add, list, 0) — bulk fold (Task #48, Wave-3 ship #4).

Predicted regime per scout doc: **mostly startup-dominated and likely
SLOWER under mamba**. `reduce` is the canonical #2100 callback-bound
hot path — every iteration crosses FFI to invoke `operator.add` from
inside the reducer. Team-lead has explicitly OK'd a Gate 2 perf FAIL
on this fixture by design; the ship contract is "surface registered,
behavior parity, document the carve".

Carve-outs documented in `functools_mod.rs`:
- reduce is #2100-bound (FFI per element); ≥1.0× CPython floor not
  expected to hold on this fixture until #2100 lands.
- lru_cache/partial/cached_property class-shell semantics are MVP
  (Instance-wrapped); they pass the conformance suite but the full
  Cache hit/miss accounting is wave-4 work.

Hoist convention (#2097): bind `reduce` and `add` to locals before
the loop so each iter is a direct call, not a per-iter module-attr
lookup.

# tier: compute
"""

import functools
import operator

_reduce = functools.reduce
_add = operator.add

DATA = list(range(10_000))
ITERS = 10

acc = 0
for _ in range(ITERS):
    acc = _reduce(_add, DATA, 0)
print("reduce_add:", acc)
