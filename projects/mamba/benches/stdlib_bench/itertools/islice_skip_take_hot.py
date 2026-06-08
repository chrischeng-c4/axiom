"""itertools.islice — skip + take perf bench.

End-user scenario: `islice(it, start, stop)` over a generator,
the canonical lazy-pagination / windowed-take primitive that
backs every chunked-decode / batched-API / sliding-window
producer. CPython's islice_next has tight start-skip + stop-
exit branches; mamba's itertools.islice currently dispatches
through a Python-level wrap.

Bounded context (DDD): stdlib_bench/itertools.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `islice` to a local before the hot loop.
"""

import sys
import time
from itertools import islice

_islice = islice

N = 1000
START = 100
STOP = 900
xs = list(range(N))
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for x in _islice(xs, START, STOP):
        acc = acc + x
_t1 = time.perf_counter()

print("islice_skip_take_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Per inner pass: sum(START..STOP-1) = (STOP-START)*(START+STOP-1)//2.
expected = ITERS * ((STOP - START) * (START + STOP - 1) // 2)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
