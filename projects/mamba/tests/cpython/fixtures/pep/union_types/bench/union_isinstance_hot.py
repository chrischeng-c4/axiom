"""Hot-loop bench for PEP 604 union types: isinstance with X|Y.

Domain: pep
Feature: union_types
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop running isinstance against a union type —
monomorphic int regime, measures PEP 604 union isinstance dispatch overhead.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)
_union = int | float | str

acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    acc ^= (1 if isinstance(v, _union) else 0) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"union_isinstance: {ITERS}")
