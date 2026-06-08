"""Hot-loop bench for builtins.bool: int→bool conversion.

Domain: builtins
Feature: bool
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop converting integers to bool (truthiness
check) — monomorphic on int input.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(100))  # 0..99, alternating truthy/falsy
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    acc ^= bool(_inputs[i % _n])

# Stdout sink — byte-equal across runtimes.
print(f"bool_int: {ITERS}")
