"""PEP 448 dict-unpack merge — `{**a, **b}` perf bench.

End-user scenario: `{**defaults, **overrides}` inside a loop,
the canonical settings-merge / kwarg-build / context-extension
idiom that backs every config layering and template-render
pipeline. CPython compiles to BUILD_MAP + DICT_UPDATE; mamba
lowers typed dict-merge to a paired bulk insert when both
sources are dict literals.

Bounded context (DDD): pep_bench/pep448_unpack_generalize.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.

The accumulator sums len() of the merged dict to keep the
per-iter body cheap.
"""

import sys
import time

defaults = {"a": 1, "b": 2, "c": 3, "d": 4}
overrides = {"c": 30, "e": 50, "f": 60}
ITERS = 100_000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    merged = {**defaults, **overrides}
    total = total + len(merged)
_t1 = time.perf_counter()

print("dict_merge_unpack_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Merged dict keys: {a, b, c, d, e, f} -> len = 6.
expected = ITERS * 6
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
