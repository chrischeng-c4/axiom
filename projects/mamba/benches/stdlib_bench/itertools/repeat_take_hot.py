"""itertools.repeat — bounded constant-stream perf bench.

End-user scenario: `zip(keys, repeat(default_value, len(keys)))` inside
a tight loop, the canonical bounded-constant-stream primitive that
backs every defaults-pair builder / placeholder column filler / fixed-
weight one-hot expander / sentinel-row materializer. CPython routes
through itertools_repeat_impl (C-level fixed-count yield loop);
mamba's itertools should hit a native impl through its typed bridge.

Distinct from `chain_two_lists_hot.py` (multi-iterable splice) and
`accumulate_hot.py` (running fold). repeat is the simplest itertools
generator — pure constant emission with a counter.

Bounded context (DDD): stdlib_bench/itertools.

Tier: compute (with per-call new-iterator + sum() reduction).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `repeat` is a module-level free fn; safe to hoist locally.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import itertools
import sys
import time

_repeat = itertools.repeat
VAL = 7
TAKE = 100
ITERS = 20000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for v in _repeat(VAL, TAKE):
        s = s + v
    acc = acc + s
_t1 = time.perf_counter()

print("repeat_take_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = VAL * TAKE
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
