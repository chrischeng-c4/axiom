"""bool(x) coercion — builtin perf bench.

End-user scenario: `bool(x)` inside a tight loop, the canonical
truthiness coercion that backs every `if x: ...`, every
short-circuit, every `flag = bool(maybe)` normalization.
CPython routes through PyObject_IsTrue + tp_bool / tp_len;
mamba's mb_bool lowers to a tag-bit + zero compare on the
small-int path.

Bounded context (DDD): builtins_bench/bool.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.

Mix of zero (falsy) and non-zero (truthy) ints so neither
branch dominates.
"""

import sys
import time

N = 1000
# Alternating zero / non-zero so bool() actually exercises both legs.
xs = [(0 if (i & 1) == 0 else i) for i in range(N)]
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for x in xs:
        if bool(x):
            acc = acc + 1
_t1 = time.perf_counter()

print("bool_truthy_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Half of [0, N) is non-zero (the odd indices, excluding i=0 which is even).
ref_per_iter = 0
for x in xs:
    if bool(x):
        ref_per_iter = ref_per_iter + 1
expected = ITERS * ref_per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
