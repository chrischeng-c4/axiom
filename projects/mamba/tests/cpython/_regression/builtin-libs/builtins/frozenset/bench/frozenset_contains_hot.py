"""Hot-loop bench for builtins.frozenset: membership test.

Domain: builtins
Feature: frozenset
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop testing integer membership in a fixed
frozenset — monomorphic on int query, static frozenset.
"""
# tier: compute


ITERS = 500_000

_fs = frozenset(range(50))  # 0..49
_queries = list(range(100))  # 0..99, half hit half miss
_n = len(_queries)

acc = 0
for i in range(ITERS):
    acc ^= int(_queries[i % _n] in _fs)

# Stdout sink — byte-equal across runtimes.
print(f"frozenset_contains: {ITERS}")
