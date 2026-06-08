"""Hot-loop bench for PEP 572 walrus: := in hot computation.

Domain: pep
Feature: walrus
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop using := to capture and reuse computed value
in the same expression — monomorphic int regime.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    if (_sq := v * v) > 100:
        acc ^= _sq & 0xFFFF
    else:
        acc ^= v & 0xFF

# Stdout sink — byte-equal across runtimes.
print(f"walrus_while: {ITERS}")
