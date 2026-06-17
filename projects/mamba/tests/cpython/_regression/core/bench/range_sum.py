# tier: required
# category: numeric
# inclusion_reason: MVP `range()` iteration baseline (10M-iter range-driven sum)

import sys
import time

_t0: int = time.perf_counter_ns()
total: int = 0
for i in range(10000000):
    total = total + i
_t1: int = time.perf_counter_ns()
print(total)
print(f"INTERNAL_TIME_NS={_t1 - _t0}", file=sys.stderr)
