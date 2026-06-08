"""Hot-loop bench for builtins.min: integer list minimum.

Domain: builtins
Feature: min
Tier: compute

# type-regime: monomorphic

End-user scenario: repeated min() over a fixed-size list of integers —
canonical monomorphic case where mamba specializes on list[int] and
can lower to a native scan loop.
"""
# tier: compute


ITERS = 50_000

# Build input outside the loop — list of int, always the same type.
_lst = list(range(100, 0, -1))  # [100, 99, ..., 1]

acc = 0
for i in range(ITERS):
    acc ^= min(_lst)

# Stdout sink — byte-equal across runtimes.
print(f"min_int_list: {ITERS}")
