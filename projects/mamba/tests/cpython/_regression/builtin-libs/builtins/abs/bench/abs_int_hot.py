"""Hot-loop bench for builtins.abs: integer absolute value.

Domain: builtins
Feature: abs
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling abs() on pre-generated negative
integers — canonical monomorphic case where mamba specializes the call
site on int and lowers to a native i64 negation branch.
"""
# tier: compute


ITERS = 500_000

# Build input outside the loop — all int, always the same type.
_inputs = [-(i % 1000 + 1) for i in range(1000)]
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    acc ^= abs(_inputs[i % _n])

# Stdout sink — byte-equal across runtimes.
print(f"abs_int: {ITERS}")
