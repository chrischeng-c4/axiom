# tier: required
# category: numeric
# inclusion_reason: MVP int arithmetic baseline (10M-iter `int += int`)

import sys
import time

_t0: int = time.perf_counter_ns()
total: int = 0
i: int = 0
while i < 10000000:
    total = total + i
    i = i + 1
_t1: int = time.perf_counter_ns()
print(total)
print(f"INTERNAL_TIME_NS={_t1 - _t0}", file=sys.stderr)
