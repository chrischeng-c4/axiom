"""Scalar per-element hot-loop bench for `math.sqrt` (Task #36).

Informational fixture — bench expects FAIL under #2100 (per-element
FFI dispatch overhead ~14us/call). Documented as the canonical
scalar-hot-loop shape so the post-#2100 win can be measured against
this baseline.

Per `feedback_mamba_perf_is_the_product` carve-out (iii): this
fixture is tracked but NOT a Phase 2 blocker. Algorithmic libs whose
hot loop calls back into mamba runtime per element are blocked on
#2100 — same regime as bisect/heapq with key=.

Hoist convention (#2097): `sqrt = math.sqrt` BEFORE the hot loop.

# tier: compute
"""

import math

# Hoist module-level attribute outside the loop (#2097).
sqrt = math.sqrt

ITERS = 100_000

acc = 0.0
for i in range(1, ITERS + 1):
    acc += sqrt(i)
print("scalar_sqrt:", int(acc))
