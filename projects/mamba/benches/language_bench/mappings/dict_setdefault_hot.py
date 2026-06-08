"""dict.setdefault — fetch-or-init perf bench.

End-user scenario: `d.setdefault(k, []).append(v)` inside a tight
loop, the canonical group-by-into-bucket primitive that backs every
ad-hoc histogram build / events-by-user collation / files-per-dir
group / rows-by-key aggregator. CPython routes through
dict_setdefault_impl (C-level single lookup + maybe-insert); mamba's
dict should hit a native impl through its typed bridge.

Bounded context (DDD): language_bench/mappings.

Tier: compute (with allocation pressure on inner lists).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: no module-level attrs to hoist; dict.setdefault is a method.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

KEYS = ["k" + str(i % 10) for i in range(100)]
# Mamba scaling is highly nonlinear past GC-threshold; small ITERS
# shows 35x slower, large ITERS (5000+) shows ~380x slower as the
# 10k GC threshold wedges. ITERS=1500 balances stable cpy wall.
ITERS = 1500

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    d = {}
    for k in KEYS:
        d.setdefault(k, []).append(1)
    s = 0
    for k in d:
        s = s + len(d[k])
    acc = acc + s
_t1 = time.perf_counter()

print("dict_setdefault_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Per-iter: 100 appends total → s = 100.
expected = ITERS * len(KEYS)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
