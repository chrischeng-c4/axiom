"""collections.deque.rotate — circular buffer shift perf bench.

End-user scenario: `d.rotate(1)` inside a tight loop over a
moderately sized deque, the canonical circular-buffer / sliding-
window shift primitive (ring queue, time-bucket carousel,
round-robin scheduler). CPython's deque.rotate runs in O(k) C;
mamba's collections.deque rotate is a Python-level slice +
re-extend over an internal list.

Bounded context (DDD): stdlib_bench/collections.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `deque` to a local before the hot loop.
"""

import sys
import time
from collections import deque

_deque = deque

N = 1000
ITERS = 10_000
d = _deque(range(N))

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    d.rotate(1)
    acc = acc + d[0]
_t1 = time.perf_counter()

print("deque_rotate_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# After k rotate(1) calls, d[0] == range(N)[-k mod N].
# Reference computes the same expected sum using a fresh deque.
ref = _deque(range(N))
ref_acc = 0
for _ in range(ITERS):
    ref.rotate(1)
    ref_acc = ref_acc + ref[0]
diff = acc - ref_acc
assert diff == 0, f"checksum mismatch: {acc} - {ref_acc} = {diff}"
