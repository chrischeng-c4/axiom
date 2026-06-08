"""Hot-loop bench for builtins.hash: hash() on integers.

Domain: builtins
Feature: hash
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop hashing integers — monomorphic on int
inputs, result XOR'd to prevent DCE.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    acc ^= hash(_inputs[i % _n])

# Stdout sink — byte-equal across runtimes.
print(f"hash_int: {ITERS}")
