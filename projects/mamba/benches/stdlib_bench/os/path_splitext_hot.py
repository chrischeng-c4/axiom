"""os.path.splitext — filename root/ext split perf bench.

End-user scenario: `splitext(path)` inside a tight loop, the
canonical filename-suffix primitive that backs every static-asset
content-type sniff / build-pipeline ext-routing / log-tail file
filter / loader-by-extension dispatch. CPython routes through pure-
Python posixpath.splitext; mamba's os.path should hit the same
algorithm through its typed bridge.

Bounded context (DDD): stdlib_bench/os.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `splitext` to a local.
"""

import sys
import time
from os.path import splitext as _splitext

EXTS = [".txt", ".jpg", ".log", ".tar.gz", ".py", ".html", ".bin"]
PATHS = [f"/var/data/tenant-{i % 10}/file-{i}{EXTS[i % len(EXTS)]}" for i in range(1000)]
ITERS = 200

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    s = 0
    for p in PATHS:
        root, ext = _splitext(p)
        s = s + len(ext)
    total = total + s
_t1 = time.perf_counter()

print("path_splitext_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for p in PATHS:
    _re = _splitext(p)
    per_iter = per_iter + len(_re[1])
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
