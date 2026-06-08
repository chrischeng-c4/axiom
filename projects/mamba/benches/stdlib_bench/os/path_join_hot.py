"""os.path.join — path-segment join perf bench.

End-user scenario: `os.path.join(base, *parts)` inside a tight loop,
the canonical filesystem path build that backs every config-file
locator / asset resolver / temp-dir scratchpad / per-tenant storage
laid-out by id. CPython routes through posixpath.join (pure Python
with separator handling); mamba's os.path should hit the same logic
through its typed bridge.

Bounded context (DDD): stdlib_bench/os.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `join` to a local before the hot loop.

Mamba quirk: `import os; os.path.join` works at module level (path
is a real submodule attribute on os, not a dotted-import), unlike
urllib.parse. Verified via probe.
"""

import os
import sys
import time

_join = os.path.join

N = 1000
parts = [(f"tenant-{i % 10}", f"year-{2020 + i % 5}", f"file-{i}.dat") for i in range(N)]
ITERS = 100

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for p in parts:
        total = total + len(_join("/var/data", p[0], p[1], p[2]))
_t1 = time.perf_counter()

print("path_join_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for p in parts:
    per_iter = per_iter + len(_join("/var/data", p[0], p[1], p[2]))
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
