"""Bulk enum.Enum creation + member lookup (Task #74, Wave-7 ship #1).

Predicted regime: heap-heavy. Each iter builds a fresh Enum class from
a 4-entry members dict + 4 EnumMember Instance allocations + a
__members__ list + dict lookups for member access. This is the per-call
Instance density family flagged in subset-B Phase 1 profile
([[project_mamba_per_call_instance_subset_b]]) — wall expected to be
soft (<=1.5x) until the F1+F2 Instance HashMap pre-size + FxHash
landings close the layered-cost gap.

The fixture is deliberately the heavy regime (not a unique() pass-through
microbench) so it pairs with future subset-B perf work: the same fixture
re-runs after F1+F2 should show a measurable wall improvement.

Workload: 1000 iters of (build Color enum + read RED.value).

Hoist convention (#2097): bind `enum.Enum` locally to avoid per-iter
module-attr lookup.
Mamba import quirk avoidance: separate `import sys` / `import time` /
`import enum` lines.

# tier: heap
"""

import enum

_Enum = enum.Enum

ITERS = 1000

acc = 0
for _ in range(ITERS):
    Color = _Enum("Color", {"RED": 1, "GREEN": 2, "BLUE": 3, "ALPHA": 4})
    # Read one member's value back so the JIT can't dead-code-eliminate
    # the class construction.
    v = Color.RED.value
    acc = acc + v
print("enum_create_bulk:", acc)
