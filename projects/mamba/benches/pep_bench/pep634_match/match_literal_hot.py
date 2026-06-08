"""PEP 634 structural pattern matching — literal arms perf bench.

End-user scenario: `match x: case 0: ... case 1: ...` over a
small literal-arm cascade, the canonical state-machine /
opcode-dispatch shape that backs every interpreter inner loop.
CPython compiles match to MATCH_CLASS / MATCH_SEQUENCE / etc.;
mamba's match lowers literal-arm chains to a jump table when
the JIT proves all arms are int literals.

Bounded context (DDD): pep_bench/pep634_match.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time

N = 1000
xs = [(i % 4) for i in range(N)]
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for x in xs:
        match x:
            case 0:
                acc = acc + 1
            case 1:
                acc = acc + 2
            case 2:
                acc = acc + 3
            case _:
                acc = acc + 4
_t1 = time.perf_counter()

print("match_literal_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Each arm fires N/4 times (since xs cycles 0..3). Per inner pass:
# (N/4)*(1+2+3+4) = (N/4)*10.
expected = ITERS * ((N // 4) * 10)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
