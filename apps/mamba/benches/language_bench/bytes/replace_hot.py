"""bytes.replace — substring-rewrite perf bench.

End-user scenario: `data.replace(old, new)` inside a tight loop, the
canonical binary-transform primitive that backs every newline
normalisation / CRLF conversion / token-rename in a protocol frame /
secret-redaction over log bytes. CPython routes through PyBytes_Replace
(C-level); mamba's bytes should hit a native impl through its typed
bridge.

Bounded context (DDD): language_bench/bytes.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: no module-level attrs to hoist; bytes.replace is a method.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

DATA = b"the quick brown fox jumps over the lazy dog " * 50
OLD = b"the"
NEW = b"THE_"
ITERS = 20000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    out = DATA.replace(OLD, NEW)
    acc = acc + len(out)
_t1 = time.perf_counter()

print("bytes_replace_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

ref = len(DATA.replace(OLD, NEW))
expected = ITERS * ref
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
