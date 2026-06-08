"""Hot-loop bench for builtins.object: object() instantiation.

Domain: builtins
Feature: object
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop creating object() instances and discarding
them — baseline cost of Python object allocation.
"""
# tier: compute


ITERS = 500_000

acc = 0
for i in range(ITERS):
    o = object()
    acc ^= id(o) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"object_create: {ITERS}")
