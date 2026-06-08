"""list.append in loop — collect-into-list mutation perf bench.

End-user scenario: build a list by repeated `append(x)` inside a tight
loop, the canonical accumulate-into-list pattern that backs every
filter-then-collect / event-buffer / log-line capture / rows-from-source
ingest before sending to a downstream consumer. CPython routes through
PyList_Append (C-level, amortized O(1) with realloc); mamba's list
should hit a native Vec impl through its typed bridge.

Bounded context (DDD): language_bench/sequences.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: no module-level attrs to hoist; list.append is a method.
"""

import sys
import time

N = 1000
# Mamba ~24x slower; cap ITERS to keep wall under ~3s.
ITERS = 200

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    out = []
    for i in range(N):
        out.append(i)
    total = total + out[N - 1]
_t1 = time.perf_counter()

print("list_append_loop_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

expected = ITERS * (N - 1)
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
