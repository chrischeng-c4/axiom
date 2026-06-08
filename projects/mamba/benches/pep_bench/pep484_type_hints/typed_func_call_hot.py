"""PEP 484 type-hint annotated function — call perf bench.

End-user scenario: `def f(x: int, y: int) -> int:` hot-called
in a tight loop, the canonical typed-function-call shape that
dominates modern Python codebases (mypy-clean services, fastapi
handlers, pydantic adapters). Annotations are stored once on
`__annotations__`, not evaluated per call in CPython; mamba
treats them as JIT type-bind hints that enable inlining.

Bounded context (DDD): pep_bench/pep484_type_hints.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time


def add_typed(x: int, y: int) -> int:
    return x + y


N = 1000
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for i in range(N):
        acc = acc + add_typed(i, i)
_t1 = time.perf_counter()

print("typed_func_call_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Per inner pass: sum(i+i for i in range(N)) = 2 * N*(N-1)//2 = N*(N-1).
expected = ITERS * (N * (N - 1))
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
