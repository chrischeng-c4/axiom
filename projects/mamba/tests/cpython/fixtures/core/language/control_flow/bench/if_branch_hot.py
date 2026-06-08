"""Hot-loop bench for language control flow: if/else branch.

Domain: language
Feature: control_flow
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop classifying integers as positive/negative/zero
via if/elif/else — monomorphic on int inputs.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(-50, 51))  # -50..50
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    if v > 0:
        acc ^= 1
    elif v < 0:
        acc ^= 2
    else:
        acc ^= 3

# Stdout sink — byte-equal across runtimes.
print(f"if_branch: {ITERS}")
