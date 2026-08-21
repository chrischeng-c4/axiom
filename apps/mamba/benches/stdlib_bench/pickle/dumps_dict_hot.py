"""pickle.dumps — small-dict serialize perf bench.

End-user scenario: `pickle.dumps(record)` inside a tight loop, the
canonical in-process snapshot path that backs every multiprocessing
arg send / shelve write / dask graph ship / disk-cache fill. CPython
routes through _pickle.Pickler with a C-level opcode emit; mamba's
pickle should ride the same compiled fast-path behind the typed
bridge — small homogeneous dicts are the realistic hot shape.

Bounded context (DDD): stdlib_bench/pickle.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `dumps` to a local before the hot loop.
"""

import pickle
import sys
import time

_dumps = pickle.dumps

N = 1000
records = [{"id": i, "name": f"row-{i}", "active": True, "score": i * 3} for i in range(N)]
ITERS = 100

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for r in records:
        total = total + len(_dumps(r))
_t1 = time.perf_counter()

print("dumps_dict_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for r in records:
    per_iter = per_iter + len(_dumps(r))
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
