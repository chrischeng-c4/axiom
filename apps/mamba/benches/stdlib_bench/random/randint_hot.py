"""random.Random.randint — bounded-int draw perf bench.

End-user scenario: `r.randint(lo, hi)` inside a tight loop, the canonical
bounded-integer-draw primitive that backs every dice roll / load-test
jitter / sharded id stamping / Monte-Carlo index pick. CPython routes
through Random.randint → Random.randrange → _randbelow; mamba's random
should hit a native impl through its typed bridge.

Bounded context (DDD): stdlib_bench/random.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: do NOT hoist `r.randint` to a local — bound-method hoist returns
None silently under mamba (same shape as the re.Pattern.search quirk).
Call `r.randint(...)` inline.

NOTE: mamba's random.Random is NOT seed-compatible with CPython's
MT19937 — same seed yields different sequences. Expected checksum is
derived from a fresh same-seeded instance, so the bench self-validates
under either runtime.

NOTE: nested `for _ in range(...)` loops misbehave under mamba (outer
`_` binding is corrupted by the inner loop), so use named loop vars
(`outer`, `inner`).
"""

import random
import sys
import time

SEED = 0xC0FFEE
HI = 1000
N = 500
ITERS = 500

# Expected sum: replay the seeded sequence once before timing.
_ref = random.Random(SEED)
expected_per_iter = 0
for _ref_i in range(N):
    expected_per_iter = expected_per_iter + _ref.randint(0, HI)

r = random.Random(SEED)

total = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    r.seed(SEED)
    s = 0
    for inner in range(N):
        s = s + r.randint(0, HI)
    total = total + s
_t1 = time.perf_counter()

print("randint_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

expected = ITERS * expected_per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
