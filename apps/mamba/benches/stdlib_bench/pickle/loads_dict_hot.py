"""pickle.loads — small-dict deserialize perf bench.

End-user scenario: `pickle.loads(blob)` inside a tight loop, the
inverse of dumps — every multiprocessing arg receive / shelve read /
dask result decode / disk-cache hit lands here. CPython routes through
_pickle.Unpickler (C-level opcode walk); mamba's pickle should hit
the same native fast-path through the typed bridge.

Bounded context (DDD): stdlib_bench/pickle.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `loads` to a local before the hot loop.
"""

import pickle
import sys
import time

_dumps = pickle.dumps
_loads = pickle.loads

N = 1000
records = [{"id": i, "name": f"row-{i}", "active": True, "score": i * 3} for i in range(N)]
blobs = [_dumps(r) for r in records]
ITERS = 100

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for b in blobs:
        total = total + len(_loads(b))
_t1 = time.perf_counter()

print("loads_dict_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Each round-trips back to a 4-field dict.
expected = ITERS * N * 4
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
