"""Hot-loop bench for builtins.id: id() on object instances.

Domain: builtins
Feature: id
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling id() on a fixed list of objects —
monomorphic on list instances.
"""
# tier: compute


ITERS = 500_000

_objects = [object() for _ in range(10)]
_n = len(_objects)

acc = 0
for i in range(ITERS):
    acc ^= id(_objects[i % _n]) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"id_obj: {ITERS}")
