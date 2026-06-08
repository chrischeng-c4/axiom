"""tuple.index() — immutable sequence search perf bench.

End-user scenario: `header_row.index("user_id")` inside a tight loop,
the canonical first-position-of primitive on an immutable sequence,
which backs every column-name to column-position resolver on a CSV
header tuple / enum-tag to variant-index converter / dispatch table
key lookup / frozen schema field-position finder. CPython routes
through tuple_index_impl (C-level linear scan with PyObject_RichCompare);
mamba's tuple should hit a native impl through its typed bridge.

Distinct from `tuple_count_hot.py` (count occurrences) and from
`list_method_index_hot.py` (mutable list variant).

Bounded context (DDD): language_bench/sequences.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `index` is a tuple method; DO NOT hoist `_idx = XS.index` —
bound-method hoist returns None silently under mamba.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

XS = ("col_a", "col_b", "col_c", "col_d", "col_e", "col_f", "col_g",
      "col_h", "col_i", "col_j", "col_k", "col_l", "col_m", "col_n",
      "col_o", "col_p", "col_q", "col_r", "col_s", "col_t")
TARGETS = ("col_a", "col_g", "col_m", "col_p", "col_t")
ITERS = 20000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for t in TARGETS:
        s = s + XS.index(t)
    acc = acc + s
_t1 = time.perf_counter()

print("tuple_method_index_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for t in TARGETS:
    per_iter = per_iter + XS.index(t)
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
