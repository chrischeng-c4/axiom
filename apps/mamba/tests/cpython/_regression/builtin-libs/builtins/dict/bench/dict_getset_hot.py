"""Hot-loop bench for builtins.dict: get and set on a fixed dict.

Domain: builtins
Feature: dict
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop doing dict.__setitem__ and __getitem__
on str keys — monomorphic on str keys, int values.
"""
# tier: compute


ITERS = 500_000

_keys = [str(i) for i in range(10)]
_n = len(_keys)

acc = 0
for i in range(ITERS):
    d = {}
    k = _keys[i % _n]
    d[k] = i
    acc ^= d[k]

# Stdout sink — byte-equal across runtimes.
print(f"dict_getset: {ITERS}")
