"""Hot-loop bench for builtins.round: round() on floats.

Domain: builtins
Feature: round
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling round(x, 2) on float values —
monomorphic on float inputs.
"""
# tier: compute


ITERS = 500_000

_inputs = [i * 0.01 for i in range(100)]  # 0.00 .. 0.99
_n = len(_inputs)

acc = 0.0
for i in range(ITERS):
    r = round(_inputs[i % _n], 2)
    acc += r
    if acc > 1000.0:
        acc -= 1000.0

# Stdout sink — byte-equal across runtimes.
print(f"round_float: {ITERS}")
