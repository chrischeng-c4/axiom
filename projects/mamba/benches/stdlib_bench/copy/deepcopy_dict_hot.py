"""copy.deepcopy — small-dict deep-copy perf bench.

End-user scenario: `copy.deepcopy(record)` inside a tight loop, the
canonical defensive-clone primitive that backs every snapshot for
rollback / safe-arg pass / immutable-view materialise. CPython routes
through copy.py (pure Python + per-type dispatch); mamba's copy hits a
known regression regime per [[project-mamba-post-2100-gc-threshold-workload-regime]]
— must keep ITERS small to avoid wedging the post-#2100 GC threshold.

Bounded context (DDD): stdlib_bench/copy.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `deepcopy` to a local before the hot loop.
"""

import copy
import sys
import time

_deepcopy = copy.deepcopy

# Small record: 4 keys, scalars + one nested list. Keep ITERS×N bounded.
N = 100
records = [{"id": i, "name": f"r-{i}", "tags": [i, i * 2, i * 3], "ok": True} for i in range(N)]
ITERS = 100  # 10k deep-copies — under the 10-30k clones/iter regime cap.

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for r in records:
        c = _deepcopy(r)
        total = total + c["id"]
_t1 = time.perf_counter()

print("deepcopy_dict_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for r in records:
    per_iter = per_iter + r["id"]
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
