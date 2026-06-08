"""float(str) — string-to-float parse builtin perf bench.

End-user scenario: `float(s)` inside a tight loop, the canonical
numeric-parse primitive that backs every CSV metric column ingest /
JSON-number decode (slow path) / config-float decode / sensor-reading
inbound. CPython routes through PyOS_string_to_double (C-level
strtod-style); mamba's float constructor should hit a native impl
through its typed bridge.

Bounded context (DDD): builtins_bench/float_parse.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `float` is a builtin; no module-attr hoisting needed.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import math
import sys
import time

SAMPLES = ["{:.4f}".format(i * 0.31 + 0.5) for i in range(100)]
ITERS = 5000

acc = 0.0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0.0
    for tok in SAMPLES:
        s = s + float(tok)
    acc = acc + s
_t1 = time.perf_counter()

print("float_from_string_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0.0
for tok in SAMPLES:
    per_iter = per_iter + float(tok)
expected = ITERS * per_iter
diff = acc - expected
assert math.isclose(acc, expected, rel_tol=1e-9), f"checksum mismatch: {acc} vs {expected} (diff={diff})"
