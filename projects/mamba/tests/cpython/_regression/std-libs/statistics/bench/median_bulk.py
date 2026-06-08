"""Sort-bounded bench for `statistics.median` (Task #41 Wave-2 ship #3).

10k-element list, sort + middle, repeated 100 times. Sorted() perf is the
bottleneck, not the wrapper. Mamba's Vec::sort_by here vs CPython's TimSort -
expected wrapper-exonerated since the sort dominates.

Predicted regime: balanced wall 2-5x, internal 0.5-1.0x.

# tier: compute
"""

import statistics

median = statistics.median

N = 10_000
DATA = [(i * 9241 + 3) & 0xFFFF for i in range(N)]  # pseudo-random ints

ITERS = 100

acc = 0
for _ in range(ITERS):
    acc += int(median(DATA))
print("stat_median_bulk:", acc)
