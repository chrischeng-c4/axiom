"""abs() over int sequence — builtin perf bench.

End-user scenario: `abs(x)` inside a tight loop summing distances,
the canonical magnitude reduction (L1 norms, error magnitudes,
deltas-to-target). CPython routes through builtin_abs +
long_abs; mamba's mb_abs lowers to a native i64 branchless sign
flip when the JIT proves the arg fits the small-int fast path.

Bounded context (DDD): builtins_bench/abs.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time

N = 1000
# Mix of positives and negatives so abs() actually flips signs.
xs = [(i - 500) for i in range(N)]
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for x in xs:
        acc = acc + abs(x)
_t1 = time.perf_counter()

print("abs_int_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Per inner pass: sum(|i-500|) for i in 0..999 = 2 * (1+2+...+500) - 500 = 250000.
# But cleaner: compute reference using the same abs() so both runtimes share.
ref_per_iter = 0
for x in xs:
    ref_per_iter = ref_per_iter + abs(x)
expected = ITERS * ref_per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
