"""Hot-loop bench for builtins.max: integer list maximum.

Domain: builtins
Feature: max
Tier: compute

# type-regime: monomorphic

End-user scenario: repeated max() over a fixed-size list of integers —
canonical monomorphic case where mamba specializes on list[int] and
lowers to a native scan loop.
"""
# tier: compute


ITERS = 50_000

# Build input outside the loop — list of int, always the same type.
_lst = list(range(1, 101))  # [1, 2, ..., 100]

acc = 0
for i in range(ITERS):
    acc ^= max(_lst)

# Stdout sink — byte-equal across runtimes.
print(f"max_int_list: {ITERS}")
