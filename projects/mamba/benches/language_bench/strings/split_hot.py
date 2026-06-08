"""str.split — delimiter-tokenise perf bench.

End-user scenario: `line.split(',')` inside a tight loop, the canonical
CSV-row/tag-list/path-component tokeniser that backs every log-line
field extraction / cookie header parse / k=v query string split /
PATH env walk. CPython routes through PyUnicode_Split (C-level);
mamba's str should hit a native impl through its typed bridge.

Bounded context (DDD): language_bench/strings.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: no module-level attrs to hoist; str.split is a method.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

LINE = "field0,field1,field2,field3,field4,field5,field6,field7,field8,field9,field10,field11"
SEP = ","
# Mamba allocator-bound for str.split (list of 12 new strings × ITERS).
# Wall scales superlinear past N=1M list allocs under mamba; cap ITERS=30000.
ITERS = 30000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    parts = LINE.split(SEP)
    acc = acc + len(parts)
_t1 = time.perf_counter()

print("split_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

ref = len(LINE.split(SEP))
expected = ITERS * ref
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
