"""Bulk-iterable bench for `statistics.stdev` (Task #41 Wave-2 ship #3).

Two-pass Welford on 1M-element float list, repeated 50 times. Predicted
regime: balanced toward compute-dominated wall 1-3x, internal 0.8-1.1x
(per scout doc - heavier inner work than fmean shifts further toward
compute-dominated). Wrapper exonerated if internal ~1.0.

Hoist `stdev = statistics.stdev` to dodge module-attr lookup (#2097).

# tier: compute
"""

import statistics

stdev = statistics.stdev

N = 1_000_000
DATA = [i * 0.5 + i * 0.0001 for i in range(N)]

ITERS = 50

acc = 0.0
for _ in range(ITERS):
    acc += stdev(DATA)
print("stat_stdev_bulk:", int(acc))
