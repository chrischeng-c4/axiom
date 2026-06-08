"""Hot-loop bench for builtins.set: add + discard cycle.

Domain: builtins
Feature: set
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop adding integers to a set and checking
membership — monomorphic on int elements.
"""
# tier: compute


ITERS = 200_000

_inputs = list(range(20))  # 20 distinct ints
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    s = set()
    for v in _inputs:
        s.add(v)
    acc ^= len(s)

# Stdout sink — byte-equal across runtimes.
print(f"set_add: {ITERS}")
