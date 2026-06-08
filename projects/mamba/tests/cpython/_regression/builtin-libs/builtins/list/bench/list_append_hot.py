"""Hot-loop bench for builtins.list: append + clear cycle.

Domain: builtins
Feature: list
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop building a list with append then clearing
it — monomorphic on int elements, exercises list growth/shrink.
"""
# tier: compute


ITERS = 100_000

_inputs = list(range(10))  # 10 elements per iter

acc = 0
for i in range(ITERS):
    lst = []
    for v in _inputs:
        lst.append(v)
    acc ^= lst[-1]

# Stdout sink — byte-equal across runtimes.
print(f"list_append: {ITERS}")
