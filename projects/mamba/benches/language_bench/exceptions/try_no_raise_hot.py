"""try/except no-raise hot-loop bench — exceptions perf.

End-user scenario: `try: dict_or_path_lookup; except KeyError: default`
inside a tight loop where the protected path is the COMMON case (no
exception raised). Backs every defensive lookup / probe-then-default
/ optional-key reader. CPython routes through SETUP_FINALLY +
POP_BLOCK (modern: lazy frame-block, near-zero overhead on no-raise);
mamba's try/except guard should be equally cheap on the happy path
through its native control-flow lowering.

This bench measures the TRY-BLOCK FIXED OVERHEAD when no exception
is raised — the canonical guard-cost probe. Distinct from raise-and-
catch benches (the slow path).

Bounded context (DDD): language_bench/exceptions.

Tier: compute (no exception alloc; pure try-block setup/teardown cost).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: try/except is syntax; no method-hoist concern.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

D = {"a": 1, "b": 2, "c": 3, "d": 4, "e": 5,
     "f": 6, "g": 7, "h": 8, "i": 9, "j": 10}
KEYS = ("a", "c", "e", "g", "i", "b", "d", "f", "h", "j")
N = 200
ITERS = 5000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for i in range(N):
        k = KEYS[i % 10]
        try:
            s = s + D[k]
        except KeyError:
            s = s + 0
    acc = acc + s
_t1 = time.perf_counter()

print("try_no_raise_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for i in range(N):
    k = KEYS[i % 10]
    per_iter = per_iter + D[k]
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
