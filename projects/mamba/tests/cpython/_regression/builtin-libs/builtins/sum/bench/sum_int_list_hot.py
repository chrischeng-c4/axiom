"""Hot-loop bench for builtins.sum: integer list summation.

Domain: builtins
Feature: sum
Tier: compute

# type-regime: monomorphic

End-user scenario: repeated sum() over a fixed-size list of integers —
canonical monomorphic workload where mamba can specialize on list[int]
and lower to a tight accumulation loop without boxing overhead.
"""
# tier: compute


ITERS = 50_000

# Build input outside the loop — list of int, always the same type.
_lst = list(range(100))

acc = 0
for i in range(ITERS):
    acc ^= sum(_lst)

# Stdout sink — byte-equal across runtimes.
print(f"sum_int_list: {ITERS}")
