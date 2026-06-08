"""Hot-loop bench for PEP 498 f-strings: building f-strings from ints.

Domain: pep
Feature: fstrings
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop constructing f-strings from integer values —
monomorphic int regime, measures PEP 498 f-string construction throughput.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    _s = f"v={v:03d}"
    acc ^= len(_s) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"fstring_build: {ITERS}")
