"""Hot-loop bench for builtins.bin: bin() on integers.

Domain: builtins
Feature: bin
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling bin() on integers in range 0..99 —
monomorphic on int inputs, result length XOR'd as sink.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    s = bin(_inputs[i % _n])
    acc ^= len(s)

# Stdout sink — byte-equal across runtimes.
print(f"bin_int: {ITERS}")
