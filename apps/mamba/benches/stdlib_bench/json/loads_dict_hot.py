"""json.loads — small-dict parse perf bench.

End-user scenario: `json.loads(s)` over a small fixed-shape
record, the canonical body-parse call that backs every HTTP
request handler / cache read / config load. CPython routes
through json/_default_decoder.decode with the C accelerator
(_json.scanstring); mamba's json.loads currently routes
through a Python-level decoder.

Bounded context (DDD): stdlib_bench/json.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `loads` to a local before the hot loop.
"""

import json
import sys
import time

_loads = json.loads

payload = '{"id":12345,"name":"widget","qty":3,"active":true,"tags":["red","small"]}'
ITERS = 100_000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    d = _loads(payload)
    total = total + len(d)
_t1 = time.perf_counter()

print("loads_dict_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Each parse yields a dict with 5 keys.
expected = ITERS * 5
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
