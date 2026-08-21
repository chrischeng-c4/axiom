"""set.union (|) — set merge perf bench.

End-user scenario: `tags | new_tags` inside a tight loop, the canonical
set-merge primitive that backs every tag-accumulator / feature-flag
roll-up / observed-keys aggregator / two-source allowlist combiner.
CPython routes through set_union_multi (C-level rehash + bucket
copy on a single new arena); mamba's set should hit a native impl
through its typed bridge.

Distinct from `intersection_hot.py` which covers `a & b` (the AND
path, output bounded by min(|a|, |b|)); union output is bounded by
|a| + |b| and exercises the new-bucket allocation path harder.

Bounded context (DDD): language_bench/sets.

Tier: compute (with new-set allocation per call).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `union`/`|` are set methods; DO NOT hoist `_u = A.union` —
bound-method hoist returns None silently under mamba.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

A = {1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29}
B = {2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 7, 9, 11}
ITERS = 20000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    u = A | B
    acc = acc + len(u)
_t1 = time.perf_counter()

print("set_union_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = len(A | B)
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
