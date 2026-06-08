"""Hot-loop bench for builtins.map: int transform via map.

Domain: builtins
Feature: map
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop applying a simple int→int function via
map() and consuming the result — monomorphic on list[int] → list[int].
"""
# tier: compute


ITERS = 50_000

# Build input outside the loop — list of int.
_lst = list(range(100))

def _double(x):
    return x * 2

acc = 0
for i in range(ITERS):
    for v in map(_double, _lst):
        acc ^= v

# Stdout sink — byte-equal across runtimes.
print(f"map_int: {ITERS}")
