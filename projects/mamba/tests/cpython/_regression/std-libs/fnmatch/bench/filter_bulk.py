"""Bulk fnmatch.filter (Task #63, Wave-5 ship #1).

Predicted regime per scout: compute (pure-string, no allocation
beyond the filtered output list). Wall target >=2.0x — string
matching with no per-call Instance allocation.

Workload: 1000-element filename list, *.py filter, 100 iters.
Per scout sequencing: fnmatch is the easiest wave-5 win because
the surface is already shipped (`fnmatch_mod.rs` 500 LOC) — this
ship just adds the perf-cross-check bench fixture + carve doc.

Hoist convention (#2097): bind `fnmatch.filter` locally.
Mamba import quirk avoidance: separate `import sys` /
`import time` / `import fnmatch` lines (Task #56 finding).

# tier: compute
"""

import fnmatch

_filter = fnmatch.filter

# Mixed extensions to exercise both PASS and FAIL match paths.
NAMES = []
for i in range(1000):
    if i % 3 == 0:
        NAMES.append(f"mod_{i}.py")
    elif i % 3 == 1:
        NAMES.append(f"mod_{i}.rs")
    else:
        NAMES.append(f"mod_{i}.txt")
PAT = "*.py"
ITERS = 100

acc = 0
for _ in range(ITERS):
    matched = _filter(NAMES, PAT)
    acc += len(matched)
print("filter_bulk:", acc)
