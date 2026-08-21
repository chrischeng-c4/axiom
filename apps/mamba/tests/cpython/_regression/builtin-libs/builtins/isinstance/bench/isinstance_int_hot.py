"""Hot-loop bench for builtins.isinstance: isinstance() on int inputs.

Domain: builtins
Feature: isinstance
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop checking isinstance(x, int) on integers —
monomorphic positive check path.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    acc ^= int(isinstance(_inputs[i % _n], int))

# Stdout sink — byte-equal across runtimes.
print(f"isinstance_int: {ITERS}")
