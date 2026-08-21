"""os.path.basename + os.path.dirname — path-split perf bench.

End-user scenario: `basename(p)` and `dirname(p)` inside a tight loop,
the canonical path-component-extract primitive that backs every log-
file rotator scanning by filename / build-artifact bucket sort by
parent dir / object-store key sharder by leaf / source-map lookup
keyed on basename. CPython routes through posixpath.basename/dirname
(Python rfind('/') logic); mamba's os.path should hit the same.

Bounded context (DDD): stdlib_bench/os.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `basename`/`dirname` via `from os.path import` because
mamba's dotted-import quirk leaves `os.path.basename` as None when
imported as `import os.path`.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time
from os.path import basename as _basename
from os.path import dirname as _dirname

PATHS = [
    "/var/log/app/2026/05/26/request-trace-{}.json".format(i)
    for i in range(50)
]
ITERS = 5000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for p in PATHS:
        s = s + len(_basename(p)) + len(_dirname(p))
    acc = acc + s
_t1 = time.perf_counter()

print("path_basename_dirname_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for p in PATHS:
    per_iter = per_iter + len(_basename(p)) + len(_dirname(p))
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
