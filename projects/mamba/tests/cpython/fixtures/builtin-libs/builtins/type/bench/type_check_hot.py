"""Hot-loop bench for builtins.type: type() one-arg dispatch.

Domain: builtins
Feature: type
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling type(obj) on integers to get
their type object — monomorphic on int inputs.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    t = type(_inputs[i % _n])
    acc ^= int(t is int)

# Stdout sink — byte-equal across runtimes.
print(f"type_check: {ITERS}")
