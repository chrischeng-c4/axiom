"""filter() over list — builtin perf bench.

End-user scenario: `list(filter(pred, xs))` retaining elements that
satisfy a predicate, the canonical functional-style selection
idiom (data cleaning, row pruning). CPython dispatches per-element
PyObject_IsTrue + PyObject_CallOneArg; mamba's filter lowers to a
fused iter that inlines the predicate dispatch.

Bounded context (DDD): builtins_bench/filter.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time


def is_even(x):
    return (x & 1) == 0


N = 1000
xs = list(range(N))
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    ys = list(filter(is_even, xs))
    total = total + len(ys)
_t1 = time.perf_counter()

print("filter_even_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Half of [0, N) satisfies the even predicate when N is even.
expected = ITERS * (N // 2)
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
