"""Hot-loop bench for builtins.complex: complex arithmetic.

Domain: builtins
Feature: complex
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop doing complex addition and multiplication
— monomorphic on complex inputs.
"""
# tier: compute


ITERS = 500_000

_inputs = [complex(i, i + 1) for i in range(10)]
_n = len(_inputs)

acc = 0.0
for i in range(ITERS):
    z = _inputs[i % _n]
    w = _inputs[(i + 1) % _n]
    r = z * w
    acc += r.real
    if acc > 1e9:
        acc -= 1e9

# Stdout sink — byte-equal across runtimes.
print(f"complex_arith: {ITERS}")
