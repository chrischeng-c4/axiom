"""Hot-loop bench for builtins.filter: integer predicate filtering.

Domain: builtins
Feature: filter
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop filtering a fixed-size int list with a
simple predicate — monomorphic on list[int], predicate always sees int.
"""
# tier: compute


ITERS = 50_000

# Build input outside the loop — list of int.
_lst = list(range(100))

def _is_even(x):
    return x % 2 == 0

acc = 0
for i in range(ITERS):
    for v in filter(_is_even, _lst):
        acc ^= v

# Stdout sink — byte-equal across runtimes.
print(f"filter_int: {ITERS}")
