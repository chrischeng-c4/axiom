"""json.dumps — small-dict serialize perf bench.

End-user scenario: `json.dumps(d)` over a small fixed-shape
dict, the canonical record-serialize call that backs every
HTTP response emitter / cache write / log line. CPython routes
through json/_default_encoder.encode with a C accelerator
(_json.encode_basestring_ascii); mamba's json.dumps currently
routes through a Python-level encoder over its own dict
iteration.

Bounded context (DDD): stdlib_bench/json.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `dumps` to a local before the hot loop.
"""

import json
import sys
import time

_dumps = json.dumps

record = {
    "id": 12345,
    "name": "widget",
    "qty": 3,
    "active": True,
    "tags": ["red", "small"],
}
ITERS = 100_000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    s = _dumps(record)
    total = total + len(s)
_t1 = time.perf_counter()

print("dumps_dict_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Every dumps emits the same string; total = ITERS * len(once).
once_len = len(_dumps(record))
expected = ITERS * once_len
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
