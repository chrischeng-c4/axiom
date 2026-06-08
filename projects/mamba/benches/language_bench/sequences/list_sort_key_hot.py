"""list.sort(key=...) — keyed-sort perf bench.

End-user scenario: `rows.sort(key=lambda r: r[1])` inside a tight loop,
the canonical secondary-sort primitive that backs every leaderboard
re-rank / top-N selection by score / event re-order by timestamp /
filename listing by mtime. CPython routes through list_sort_impl with
a Python-level key callback (PyObject_Call per element); mamba's list
should hit a native sort through its typed bridge, with the key call
crossing the typed bridge.

Bounded context (DDD): language_bench/sequences.

Tier: compute (with per-element Python callable invocation).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: no module-level attrs to hoist; list.sort is a method.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time


def _key(row):
    return row[1]


BASE = [("k" + str(i % 23), (i * 31 + 7) % 1000) for i in range(200)]
ITERS = 8000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    rows = list(BASE)
    rows.sort(key=_key)
    acc = acc + rows[0][1] + rows[-1][1]
_t1 = time.perf_counter()

print("list_sort_key_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

ref = list(BASE)
ref.sort(key=_key)
per_iter = ref[0][1] + ref[-1][1]
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
