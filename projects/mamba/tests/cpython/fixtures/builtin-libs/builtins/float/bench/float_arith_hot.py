"""Hot-loop bench for builtins.float: floating-point arithmetic.

Domain: builtins
Feature: float
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop doing float addition and multiplication
on small floats — monomorphic on float, JIT-specializable.
"""
# tier: compute


ITERS = 500_000

_inputs = [float(i) * 0.01 for i in range(100)]  # 0.0 .. 0.99
_n = len(_inputs)

acc = 0.0
for i in range(ITERS):
    acc += _inputs[i % _n]
    if acc > 1_000_000.0:
        acc -= 1_000_000.0

# Stdout sink — byte-equal across runtimes.
print(f"float_arith: {ITERS}")
