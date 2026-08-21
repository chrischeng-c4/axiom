"""fnmatch.fnmatch — shell-glob match perf bench.

End-user scenario: `fnmatch(name, pattern)` inside a tight loop, the
canonical filename-filter primitive that backs every gitignore-walker /
log-file selector / artifact-collector / config-include scanner. CPython
routes through fnmatch (pure Python + translate→re.compile cached);
mamba's fnmatch should hit the same algorithm through its typed bridge.

Bounded context (DDD): stdlib_bench/fnmatch.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `fnmatch` to a local before the hot loop.
"""

import fnmatch
import sys
import time

_fnmatch = fnmatch.fnmatch

N = 1000
names = [f"file-{i:05d}.{('py' if i % 3 == 0 else 'txt' if i % 3 == 1 else 'log')}" for i in range(N)]
PATTERN = "file-*.py"
ITERS = 100

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for n in names:
        if _fnmatch(n, PATTERN):
            total = total + 1
_t1 = time.perf_counter()

print("fnmatch_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for n in names:
    if _fnmatch(n, PATTERN):
        per_iter = per_iter + 1
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
