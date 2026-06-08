"""Hot-loop bench for builtins.int: integer arithmetic.

Domain: builtins
Feature: int
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop doing integer addition and multiplication
on small ints — monomorphic on int, JIT-specializable.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(100))  # 0..99
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    x = _inputs[i % _n]
    acc = (acc + x) % 1_000_000

# Stdout sink — byte-equal across runtimes.
print(f"int_arith: {ITERS}")
