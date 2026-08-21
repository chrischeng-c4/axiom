"""Hot-loop bench for builtins.reversed: list reverse iteration.

Domain: builtins
Feature: reversed
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop creating a reversed iterator over a fixed
list and consuming all elements — monomorphic on list[int].
"""
# tier: compute


ITERS = 50_000

# Build input outside the loop — list of int.
_lst = list(range(100))

acc = 0
for i in range(ITERS):
    for v in reversed(_lst):
        acc ^= v

# Stdout sink — byte-equal across runtimes.
print(f"reversed_list: {ITERS}")
