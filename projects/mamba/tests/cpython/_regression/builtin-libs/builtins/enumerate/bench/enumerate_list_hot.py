"""Hot-loop bench for builtins.enumerate: list enumeration.

Domain: builtins
Feature: enumerate
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop enumerating a fixed-size int list,
accumulating index+value products — monomorphic on list[int].
"""
# tier: compute


ITERS = 50_000

# Build input outside the loop — list of int.
_lst = list(range(100))

acc = 0
for i in range(ITERS):
    for idx, v in enumerate(_lst):
        acc ^= idx + v

# Stdout sink — byte-equal across runtimes.
print(f"enumerate_list: {ITERS}")
