"""PEP 557 @dataclass — instantiate + attribute-read perf bench.

End-user scenario: `@dataclass class Point: x: int; y: int`
instantiated repeatedly with `p.x` / `p.y` reads inside the
loop — the canonical typed-record-instance shape that backs
every dto / row / view-model in modern Python services.
CPython routes through __init__ + type.__call__; mamba lowers
typed dataclass instances to a struct the JIT can sometimes
keep stack-resident.

Bounded context (DDD): pep_bench/pep557_dataclasses.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time
from dataclasses import dataclass


@dataclass
class Point:
    x: int
    y: int


N = 1000
ITERS = 100

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for i in range(N):
        p = Point(i, i + 1)
        acc = acc + p.x + p.y
_t1 = time.perf_counter()

print("dataclass_attr_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Per inner pass: sum(i + (i+1) for i in range(N)) = 2*sum(0..N-1) + N
# = N*(N-1) + N = N*N.
expected = ITERS * (N * N)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
