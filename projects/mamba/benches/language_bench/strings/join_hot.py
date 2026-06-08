"""str.join — sep-joined concat perf bench.

End-user scenario: `sep.join(parts)` inside a tight loop, the canonical
collect-and-emit primitive that backs every CSV-row format / URL query
string build / shell-argv print / SQL IN-clause assembly. CPython
routes through PyUnicode_Join (C-level two-pass: measure + copy);
mamba's str should hit a native impl through its typed bridge.

Bounded context (DDD): language_bench/strings.

Tier: compute (with allocation of the joined string per call).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: no module-level attrs to hoist; str.join is a method.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

PARTS = ["field" + str(i) for i in range(20)]
SEP = ","
ITERS = 500000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    joined = SEP.join(PARTS)
    acc = acc + len(joined)
_t1 = time.perf_counter()

print("str_join_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

expected = ITERS * len(SEP.join(PARTS))
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
