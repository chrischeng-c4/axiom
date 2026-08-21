"""list.index() — sequential search method perf bench.

End-user scenario: `xs.index(target)` inside a tight loop, the
canonical first-position-of primitive that backs every enum-value to
index converter / parser dispatch-table resolver / column-name to
column-position resolver / replace-by-value index finder. CPython
routes through list_index_impl (C-level linear scan with
PyObject_RichCompare); mamba's list should hit a native impl through
its typed bridge.

Distinct from `list_index_hot.py` which covers `xs[i]` subscript reads.

Bounded context (DDD): language_bench/sequences.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `index` is a list method; DO NOT hoist `_idx = XS.index` —
bound-method hoist returns None silently under mamba.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

XS = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150,
      160, 170, 180, 190, 200, 210, 220, 230, 240, 250]
TARGETS = [10, 80, 130, 200, 250]
ITERS = 20000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for t in TARGETS:
        s = s + XS.index(t)
    acc = acc + s
_t1 = time.perf_counter()

print("list_method_index_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for t in TARGETS:
    per_iter = per_iter + XS.index(t)
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
