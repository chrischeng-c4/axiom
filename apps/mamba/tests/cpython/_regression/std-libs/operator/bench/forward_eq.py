"""Forward-wrapper hot-loop bench for `operator` (Task #39 — Wave-2 #2).

End-user scenario: bulk `operator.eq` calls inside an accumulator loop. This
is the canonical startup-dominated tier for a thin-wrapper shim — each call
crosses the FFI boundary, does a single comparison via the runtime
`mb_eq` primitive, and returns a boxed bool. Per-call cost is on the
order of a few JIT-emitted instructions; total wall is dominated by
Python startup amortization (see cross_runtime.rs caveat).

Hoist convention (per #2097): module-level attributes are hoisted to
locals BEFORE the hot loop. Without hoisting, mamba's module-attr
lookup at the call site is ~5x slower than the hoisted form.

#2105 avoidance: no `assert` between the hot-loop call and the next
statement that depends on it. The post-loop `print` of `acc` happens

# tier: startup
"""

import operator

# Hoist module-level attributes outside the loop (#2097).
eq = operator.eq
add = operator.add

ITERS = 100_000

acc = 0
for i in range(ITERS):
    # eq returns True/False; bool → int gives 0/1, summed into acc.
    if eq(i & 7, 0):
        acc = add(acc, 1)
print("operator_forward_eq:", acc)
