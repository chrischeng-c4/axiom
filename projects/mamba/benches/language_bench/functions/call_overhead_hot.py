"""Function call overhead hot-loop bench — language-core function perf.

End-user scenario: tight loop calling a tiny user-defined Python
function. Measures pure call-frame setup + arg-binding + return,
the inner-cost of every helper-heavy codebase. Mamba's JIT inlines
small monomorphic call sites where possible; CPython always pays
the CALL_FUNCTION bytecode + PyFrame setup cost.

Bounded context (DDD): language_bench/functions — first member of
the language-core function perf suite.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time


def inc(x):
    return x + 1


ITERS = 1_000_000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    acc = inc(acc)
_t1 = time.perf_counter()

print("call_overhead_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# acc starts at 0 and gets +1 per call; after ITERS calls it equals ITERS.
diff = acc - ITERS
assert diff == 0, f"checksum mismatch: {acc} - {ITERS} = {diff}"
