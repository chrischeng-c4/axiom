"""bytes.find — substring-scan perf bench.

End-user scenario: `data.find(needle)` inside a tight loop, the canonical
binary substring-locate primitive that backs every HTTP header sniff /
log-line tag scan / protocol delimiter probe / framed-buffer parser.
CPython routes through PyBytes_Find (C-level memmem-style); mamba's
bytes should hit a native impl through its typed bridge.

Bounded context (DDD): language_bench/bytes.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: no module-level attrs to hoist; bytes.find is a method.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

DATA = b"the quick brown fox jumps over the lazy dog " * 50
NEEDLE = b"lazy"
ITERS = 5000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    pos = DATA.find(NEEDLE)
    while pos != -1:
        s = s + pos
        pos = DATA.find(NEEDLE, pos + 1)
    acc = acc + s
_t1 = time.perf_counter()

print("bytes_find_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
pos = DATA.find(NEEDLE)
while pos != -1:
    per_iter = per_iter + pos
    pos = DATA.find(NEEDLE, pos + 1)
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
