"""PEP 572 walrus operator (assignment expression) — perf bench.

End-user scenario: `while (n := next_n()) > 0: ...` — the
canonical assign-and-test idiom for stream draining /
chunk-decode loops. CPython compiles `:=` to STORE_FAST +
LOAD_FAST; mamba folds the assignment into the comparator.

Bounded context (DDD): pep_bench/pep572_walrus.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.

Uses a manual countdown so the walrus actually controls the
loop exit — `while (n := n - 1) > 0` is the textbook PEP 572
example.
"""

import sys
import time

START = 1000
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    n = START
    while (n := n - 1) > 0:
        acc = acc + n
_t1 = time.perf_counter()

print("walrus_while_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Per outer iter: sum(n for n in range(START - 1, 0, -1)) = (START-1)*START//2.
expected = ITERS * ((START - 1) * START // 2)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
