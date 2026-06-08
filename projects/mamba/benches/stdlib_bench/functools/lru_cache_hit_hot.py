"""functools.lru_cache — memoization-hit perf bench.

End-user scenario: `@lru_cache(maxsize=128)` on a pure fn called with
a small repeating arg set, the canonical memoization-hit primitive
that backs every cached request-router classifier / cached fast-path
parser / repeated normalize-and-tag / cached dimensional analysis on
recurring inputs. CPython routes through lru_cache_wrapper (C-level
hash + linked-list bump on hit, fast path); mamba's functools should
hit a native impl through its typed bridge.

This bench measures STEADY-STATE HIT cost (warm cache), not miss /
eviction. The fn body is intentionally trivial so the result isolates
the wrapper overhead, not the inner work.

Bounded context (DDD): stdlib_bench/functools.

Tier: compute (with per-call hash + dict probe + result reuse).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `@lru_cache` returns a wrapped callable — it's a plain fn
under both runtimes; safe to hoist locally.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import functools
import sys
import time


@functools.lru_cache(maxsize=128)
def double(n):
    return n + n


KEYS = (1, 7, 3, 11, 5, 13, 2, 17, 9, 19, 4, 23, 6, 29, 8, 31)
ITERS = 30000

# Prime the cache so the loop measures only the hit-path.
for k in KEYS:
    double(k)

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for k in KEYS:
        s = s + double(k)
    acc = acc + s
_t1 = time.perf_counter()

print("lru_cache_hit_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for k in KEYS:
    per_iter = per_iter + double(k)
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
