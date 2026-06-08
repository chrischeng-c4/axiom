"""PEP 343 `with` statement — context-manager perf bench.

End-user scenario: `with cm: ...` inside a tight loop, the
canonical scope-bounded acquire/release primitive that backs
every lock acquire, every transaction, every typed
suppression. CPython compiles to BEFORE_WITH + WITH_EXCEPT_START
+ POP_BLOCK; mamba lowers to a try-finally with __enter__ /
__exit__ dispatch the JIT can sometimes elide when both
methods are trivial.

Bounded context (DDD): pep_bench/pep343_with.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.

The CM is intentionally trivial (no side effects in
__enter__/__exit__) so the measurement isolates context-block
dispatch overhead.
"""

import sys
import time


class NullCM:
    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb):
        return False


N = 1000
ITERS = 1000
cm = NullCM()

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for i in range(N):
        with cm:
            acc = acc + i
_t1 = time.perf_counter()

print("with_ctx_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Per inner pass: sum(range(N)) = N*(N-1)//2.
expected = ITERS * (N * (N - 1) // 2)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
