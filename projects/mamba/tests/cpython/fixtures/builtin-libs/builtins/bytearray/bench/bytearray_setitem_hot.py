"""Hot-loop bench for builtins.bytearray: in-place mutation via setitem.

Domain: builtins
Feature: bytearray
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop mutating individual bytes in a bytearray —
monomorphic on bytearray with int index and int value.
"""
# tier: compute


ITERS = 200_000

_ba = bytearray(100)  # 100 zero bytes
_n = len(_ba)

acc = 0
for i in range(ITERS):
    idx = i % _n
    _ba[idx] = (idx * 3) & 0xFF
    acc ^= _ba[idx]

# Stdout sink — byte-equal across runtimes.
print(f"bytearray_setitem: {ITERS}")
