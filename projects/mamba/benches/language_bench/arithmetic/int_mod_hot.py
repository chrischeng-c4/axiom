"""Integer modulo (%) hot-loop bench — arithmetic perf.

End-user scenario: tight loop over `n % d` integer mod operations, the
foundation of every ring-buffer index wrap (`i % capacity`) / shard-
key selector (`hash(k) % nshards`) / round-robin slot picker (`turn
% nslots`) / leap-year predicate (`y % 4`). CPython routes through
long_remainder (C-level multi-precision long mod + sign-correct);
mamba's int mod should hit a native i64 rem path through its typed
bridge.

Distinct from `int_div_hot.py` (//) — same CPU divide instruction
under the hood but returns the remainder; on some pipelines a
single divrem covers both, on others mod is a separate path with
its own sign-fix-up cost.

Bounded context (DDD): language_bench/arithmetic.

Tier: compute (with branch on negative-operand for mod-floor semantics).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `%` is a syntax op — no hoisting concern.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

N = 1000
ITERS = 5000
DIVISOR = 7

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for i in range(N):
        s = s + (i % DIVISOR)
    acc = acc + s
_t1 = time.perf_counter()

print("int_mod_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for i in range(N):
    per_iter = per_iter + (i % DIVISOR)
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
