"""frozenset.intersection (&) — immutable set AND perf bench.

End-user scenario: `allowed_caps & user_caps` on frozen permission
sets inside a tight loop, the canonical immutable-set AND primitive
that backs every cached-allowlist check / dispatch-capability test /
frozen-tag intersect-with-event-tags / RBAC permission overlap check.
CPython routes through frozenset_intersection (C-level probe of
smaller into larger + new frozenset build); mamba's frozenset should
hit a native impl through its typed bridge.

Distinct from `intersection_hot.py` which covers mutable `set & set`;
frozenset hashes the resulting container as well, exercising a
different code path on both runtimes.

Bounded context (DDD): language_bench/sets.

Tier: compute (with new-frozenset allocation per call).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `intersection`/`&` are frozenset methods; DO NOT hoist
`_i = A.intersection` — bound-method hoist returns None silently
under mamba.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

A = frozenset({1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29})
B = frozenset({2, 3, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35, 38})
ITERS = 20000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    inter = A & B
    acc = acc + len(inter)
_t1 = time.perf_counter()

print("frozenset_intersection_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = len(A & B)
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
