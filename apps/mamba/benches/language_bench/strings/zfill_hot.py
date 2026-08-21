"""str.zfill — left-zero-pad-to-width perf bench.

End-user scenario: `str(n).zfill(8)` inside a tight loop, the canonical
numeric-string padder that backs every sortable filename builder
(`page_00042.html`) / sequence-number formatter / fixed-width log id
emitter / hex-byte stringifier helper. CPython routes through
unicode_zfill (C-level memcpy + memset on a single arena); mamba's str
should hit a native impl through its typed bridge.

Bounded context (DDD): language_bench/strings.

Tier: compute (with new-string allocation per call).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `zfill` is a str method; no module-attr hoisting needed.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

WIDTH = 8
N = 100
ITERS = 5000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for i in range(N):
        s = s + len(str(i).zfill(WIDTH))
    acc = acc + s
_t1 = time.perf_counter()

print("zfill_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for i in range(N):
    per_iter = per_iter + len(str(i).zfill(WIDTH))
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
