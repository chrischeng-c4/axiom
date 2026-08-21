"""Integer abs/neg unary-op hot-loop bench — int unary perf.

End-user scenario: tight loop over `abs(x)` and `-x` integer unary
ops, the foundation of every signed-difference magnitude
(`abs(a - b)`) / sign-flip iteration (`-i`) / distance-from-zero
metric / signed-pixel-offset normalizer. CPython routes through
long_abs / long_neg (C-level sign-bit flip + new-PyLong); mamba's
int should hit a native i64 neg path through its typed bridge.

abs and neg pair is the cheapest unary-op family — single sign bit
manipulation. Wins on this bench reflect tight-loop fusion quality,
not algorithmic improvements.

Bounded context (DDD): language_bench/integers.

Tier: compute (per-call new-PyLong on CPython; native i64 on mamba).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `abs` is a builtin; `-` is a syntax op — no hoisting concern
(would-be-hoist of `_abs = abs` is safe — `abs` is a module-level
free fn, not a bound method).

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

_abs = abs
N = 1000
ITERS = 5000
CENTER = 500

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for i in range(N):
        s = s + _abs(i - CENTER) + (-i)
    acc = acc + s
_t1 = time.perf_counter()

print("abs_neg_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for i in range(N):
    per_iter = per_iter + _abs(i - CENTER) + (-i)
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
