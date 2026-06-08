"""PEP 526 variable annotations — perf bench.

End-user scenario: `x: int = 5` at module / function scope, the
canonical annotated-assignment form pervasive in modern typed
code. At module scope, the annotation lands in __annotations__
each pass; in function scope it is bytecode-eliminated by
CPython but still computed once on entry. CPython compiles to
STORE_NAME (no SETUP_ANNOTATIONS in function bodies); mamba
treats the annotation as a typed-bind hint that the JIT folds
into a direct local slot.

Bounded context (DDD): pep_bench/pep526_var_annot.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.

The bench measures the annotated-assignment path by re-binding
typed locals inside a function body — the dominant production
pattern.
"""

import sys
import time


def annotated_sum(n: int) -> int:
    total: int = 0
    i: int = 0
    while i < n:
        total = total + i
        i = i + 1
    return total


N = 1000
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    acc = acc + annotated_sum(N)
_t1 = time.perf_counter()

print("annot_assign_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Per outer iter: sum(range(N)) = N*(N-1)//2.
expected = ITERS * (N * (N - 1) // 2)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
