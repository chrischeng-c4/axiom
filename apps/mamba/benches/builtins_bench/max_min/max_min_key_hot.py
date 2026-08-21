"""min/max with key= callback — keyed-extremum builtin perf bench.

End-user scenario: `min(rows, key=lambda r: r[1])` inside a tight
loop, the canonical keyed-extremum primitive that backs every
top-row pick after multi-field sort / largest-by-score selector /
nearest-by-distance argmin / lowest-mtime file finder. CPython
routes through builtin_min_max_impl (C-level fold + PyObject_Call
per element); mamba's builtins should hit a native impl + typed
bridge for the callback.

Bounded context (DDD): builtins_bench/max_min.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `min`/`max` are builtins; no module-attr hoisting needed.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time


def _key(row):
    return row[1]


ROWS = [(i, (i * 13 + 7) % 1000) for i in range(200)]
ITERS = 5000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    lo = min(ROWS, key=_key)
    hi = max(ROWS, key=_key)
    acc = acc + lo[1] + hi[1]
_t1 = time.perf_counter()

print("max_min_key_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

ref_lo = min(ROWS, key=_key)
ref_hi = max(ROWS, key=_key)
per_iter = ref_lo[1] + ref_hi[1]
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
