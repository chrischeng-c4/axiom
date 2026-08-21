"""Hot-loop bench for builtins.format: format() on integers.

Domain: builtins
Feature: format
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling format(n, 'd') on integers —
monomorphic on int inputs with fixed spec string.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    s = format(_inputs[i % _n], "d")
    acc ^= len(s)

# Stdout sink — byte-equal across runtimes.
print(f"format_int: {ITERS}")
