"""Hot-loop bench for builtins.issubclass: issubclass() on builtin types.

Domain: builtins
Feature: issubclass
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling issubclass(cls, object) on builtin
types — monomorphic positive check path.
"""
# tier: compute


ITERS = 500_000

_classes = [int, float, str, list, dict, tuple, set, frozenset, bool, bytes]
_n = len(_classes)

acc = 0
for i in range(ITERS):
    acc ^= int(issubclass(_classes[i % _n], object))

# Stdout sink — byte-equal across runtimes.
print(f"issubclass: {ITERS}")
