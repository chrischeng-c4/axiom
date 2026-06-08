"""random.random — Mersenne Twister float draw perf bench.

End-user scenario: `random.random()` inside a tight loop, the
canonical uniform-[0,1) draw that backs every Monte Carlo
estimator / synthetic-data generator / dithering pass. CPython
routes through random_random with a C-level Mersenne Twister;
mamba's random.random calls into the same MT state machine
through its own typed bridge.

Bounded context (DDD): stdlib_bench/random.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `random` to a local; seed for determinism.
"""

import math
import random
import sys
import time

# Seed both runtimes identically so the per-draw sequence matches.
random.seed(42)
_random = random.random

ITERS = 1_000_000

acc = 0.0
_t0 = time.perf_counter()
for _ in range(ITERS):
    acc = acc + _random()
_t1 = time.perf_counter()

print("random_float_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Reseed + recompute the reference for the same sequence.
random.seed(42)
ref = 0.0
for _ in range(ITERS):
    ref = ref + random.random()
assert math.isclose(acc, ref, rel_tol=1e-9, abs_tol=1e-3), (
    f"checksum mismatch: {acc} vs {ref}"
)
