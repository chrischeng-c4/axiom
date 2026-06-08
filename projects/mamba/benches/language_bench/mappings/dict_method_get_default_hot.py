"""dict.get(key, default) — defensive lookup perf bench.

End-user scenario: `cfg.get("timeout_ms", 5000)` inside a tight loop,
the canonical key-or-fallback primitive that backs every config
defaults reader / sparse JSON traverser / feature-flag check with
disabled-default / cache-hit-or-zero metric increment. CPython routes
through dict_get_impl (C-level open-addressed probe + default branch);
mamba's dict should hit a native impl through its typed bridge.

Bench mixes hit and miss keys 50/50 to exercise both the found-fast
path and the missing-fall-through-to-default path.

Distinct from `dict_get_hot.py` which covers `d[k]` subscript reads.

Bounded context (DDD): language_bench/mappings.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `get` is a dict method; DO NOT hoist `_get = d.get` — bound-
method hoist returns None silently under mamba.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

D = {"k" + str(i): i for i in range(50)}
PROBES = ["k0", "miss_a", "k10", "miss_b", "k25", "miss_c", "k40", "miss_d",
          "k5", "miss_e", "k15", "miss_f", "k30", "miss_g", "k45", "miss_h"]
DEFAULT = -1
ITERS = 30000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for k in PROBES:
        s = s + D.get(k, DEFAULT)
    acc = acc + s
_t1 = time.perf_counter()

print("dict_method_get_default_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for k in PROBES:
    per_iter = per_iter + D.get(k, DEFAULT)
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
