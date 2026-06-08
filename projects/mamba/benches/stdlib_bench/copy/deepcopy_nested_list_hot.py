"""copy.deepcopy — nested-list deep-copy perf bench.

End-user scenario: `copy.deepcopy(grid)` for a 2D numeric matrix inside
a tight loop, the canonical recursive deep-copy primitive for nested
sequences that backs every undo-stack snapshot of a 2D state /
simulation step-isolation copy / matrix-row separate-eval pre-step.
CPython routes through copy._deepcopy_atomic + _deepcopy_list (Python-
level walk with memo dict); mamba's copy should hit a native impl
through its typed bridge.

Distinct from `deepcopy_dict_hot.py` (dict-of-dicts shape). Nested list
exercises the list-recurse path + the memo-cycle-detect on identical
inner-list IDs.

Bounded context (DDD): stdlib_bench/copy.

Tier: compute (with per-call new-memo-dict + per-cell new-int boxing).
ITERS deliberately low — post-#2100 GC threshold (10k clones/iter)
wedges high-cadence workloads (memo: post_2100_gc_threshold_workload).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `deepcopy` is a module-level free fn; safe to hoist locally.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import copy
import sys
import time

_deepcopy = copy.deepcopy
# 4x4 numeric grid — kept small to stay well under the post-#2100 GC
# threshold (~10k clones/iter total cap).
GRID = [[1, 2, 3, 4],
        [5, 6, 7, 8],
        [9, 10, 11, 12],
        [13, 14, 15, 16]]
ITERS = 2000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    g = _deepcopy(GRID)
    acc = acc + len(g) + len(g[0])
_t1 = time.perf_counter()

print("deepcopy_nested_list_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Each iter contributes len(GRID) + len(GRID[0]) = 4 + 4 = 8
per_iter = len(GRID) + len(GRID[0])
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
