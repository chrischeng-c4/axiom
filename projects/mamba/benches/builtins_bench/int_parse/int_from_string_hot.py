"""int(str) — string-to-int parse builtin perf bench.

End-user scenario: `int(s)` inside a tight loop, the canonical
numeric-parse primitive that backs every CSV column ingest / log line
counter extract / config-int decode / query-param coerce. CPython
routes through PyLong_FromString (C-level base-10 digit loop); mamba's
int constructor should hit a native impl through its typed bridge.

Bounded context (DDD): builtins_bench/int_parse.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `int` is a builtin; no module-attr hoisting needed.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

SAMPLES = [str(i) for i in range(0, 1000, 7)]
ITERS = 5000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for tok in SAMPLES:
        s = s + int(tok)
    acc = acc + s
_t1 = time.perf_counter()

print("int_from_string_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for tok in SAMPLES:
    per_iter = per_iter + int(tok)
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
