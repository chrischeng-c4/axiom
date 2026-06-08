"""json.dumps — nested-list serialize perf bench.

End-user scenario: `json.dumps(matrix)` for a small numeric grid inside
a tight loop, the canonical numeric-array serialize primitive that backs
every chart-data emitter / sparse table dump / coordinate-list API
response / metrics-window payload writer. CPython routes through
py_encode_basestring_ascii + encoder_listencode_list (C-level walk +
buffer-grow + ascii-encode); mamba's json should hit a native impl
through its typed bridge.

Distinct from `dumps_dict_hot.py` which exercises the dict-encode path
(key-sort + colon emit + escape); list-encode is brackets + commas +
recursive walk, a different hot inner loop.

Bounded context (DDD): stdlib_bench/json.

Tier: compute (with new-string allocation per call).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `dumps` is a module-level free fn; safe to hoist locally.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import json
import sys
import time

_dumps = json.dumps
# 4x4 numeric grid — small enough that per-call serialize dominates
# over per-row walk cost; representative of widget data + API payloads.
GRID = [[1, 2, 3, 4],
        [5, 6, 7, 8],
        [9, 10, 11, 12],
        [13, 14, 15, 16]]
ITERS = 20000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = _dumps(GRID)
    acc = acc + len(s)
_t1 = time.perf_counter()

print("dumps_nested_list_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = len(json.dumps(GRID))
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
