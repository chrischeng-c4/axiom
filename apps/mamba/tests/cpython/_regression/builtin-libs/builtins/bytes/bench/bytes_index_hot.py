"""Hot-loop bench for builtins.bytes: indexing into a fixed bytes object.

Domain: builtins
Feature: bytes
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop indexing bytes to get integer values —
monomorphic on bytes input, result is int.
"""
# tier: compute


ITERS = 500_000

_data = bytes(range(100))  # 100 bytes: 0x00..0x63
_n = len(_data)

acc = 0
for i in range(ITERS):
    acc ^= _data[i % _n]

# Stdout sink — byte-equal across runtimes.
print(f"bytes_index: {ITERS}")
