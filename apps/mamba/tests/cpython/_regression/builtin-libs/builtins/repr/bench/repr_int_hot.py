"""Hot-loop bench for builtins.repr: repr() on integers.

Domain: builtins
Feature: repr
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling repr() on int values —
monomorphic on int inputs, result is discarded after XOR of length.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    s = repr(_inputs[i % _n])
    acc ^= len(s)

# Stdout sink — byte-equal across runtimes.
print(f"repr_int: {ITERS}")
