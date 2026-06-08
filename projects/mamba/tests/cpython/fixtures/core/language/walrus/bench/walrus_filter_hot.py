"""Hot-loop bench for language walrus: filter+transform in comprehension.

Domain: language
Feature: walrus
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop using walrus operator to filter and capture
values in a list comprehension — monomorphic int regime.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    # walrus: compute once, compare, use result
    if (_sq := v * v) > 100:
        acc ^= _sq & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"walrus_filter: {ITERS}")
