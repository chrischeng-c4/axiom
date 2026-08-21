"""Hot-loop bench for builtins.iter: list iterator creation + consumption.

Domain: builtins
Feature: iter
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling iter() on a fixed list and
consuming elements via next() — tests iterator protocol overhead.
"""
# tier: compute


ITERS = 100_000

_lst = list(range(50))

acc = 0
for i in range(ITERS):
    _it = iter(_lst)
    for v in _it:
        acc ^= v

# Stdout sink — byte-equal across runtimes.
print(f"iter_list: {ITERS}")
