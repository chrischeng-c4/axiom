"""json.loads — nested-list parse perf bench.

End-user scenario: `loads(json_text)` of a nested-array document
inside a tight loop, the canonical config-load primitive that backs
every WebSocket frame parser / metrics ingest / array-of-arrays
config / matrix-from-disk reload. CPython routes through C-level
_json.scanner; mamba's json should hit a native impl through its
typed bridge.

Bounded context (DDD): stdlib_bench/json.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `loads` to a local.
"""

import json
import sys
import time

_loads = json.loads

# 50 rows × 20 items
PAYLOAD = "[" + ",".join("[" + ",".join(str(j) for j in range(20)) + "]" for _ in range(50)) + "]"
ITERS = 200

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    parsed = _loads(PAYLOAD)
    s = 0
    for row in parsed:
        s = s + len(row)
    total = total + s
_t1 = time.perf_counter()

print("loads_nested_list_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# 50 rows of 20 items = 1000 per iter
expected = ITERS * 50 * 20
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
