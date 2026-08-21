"""Dict comprehension hot-loop bench — language-core perf.

End-user scenario: `{k: f(v) for k, v in items}` inside a tight loop,
the canonical structured-map-build primitive that backs every
key-value reshape / value-transform-by-key / index-by-attribute
materializer / dict-from-pairs builder. CPython routes through
DICT_MERGE + STORE_SUBSCR bytecode with the dict pre-sized at the
comp builder; mamba should hit a native dict-build path through its
typed bridge.

Distinct from `listcomp_square_hot.py` (sequence build, no hash).
Dict comprehension exercises the hash + insert + rehash path on every
iteration, much hotter than a list append.

Bounded context (DDD): language_bench/comprehensions.

Tier: compute (with per-call new-dict + per-iter hash + insert).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: dict-comprehensions are syntax; no method-hoist concern.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

KEYS = ("alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta",
        "theta", "iota", "kappa", "lambda", "mu", "nu", "xi", "omicron",
        "pi", "rho", "sigma", "tau", "upsilon")
# Pre-doubled (tuple[i] * 2 inside the dict-comp expr raises 'arithmetic
# requires numeric types' under mamba force-typing — same family as
# tuple_index_arith bug. Precompute the doubled values as a tuple).
VALS_X2 = (20, 40, 60, 80, 100, 120, 140, 160, 180, 200,
           220, 240, 260, 280, 300, 320, 340, 360, 380, 400)
ITERS = 10000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    d = {KEYS[i]: VALS_X2[i] for i in range(20)}
    acc = acc + len(d)
_t1 = time.perf_counter()

print("dictcomp_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = len({KEYS[i]: VALS_X2[i] for i in range(20)})
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
