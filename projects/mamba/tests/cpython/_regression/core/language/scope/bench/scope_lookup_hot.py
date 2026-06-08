"""Hot-loop bench for language scope: local variable lookup.

Domain: language
Feature: scope
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop reading local variables in a function —
monomorphic int regime, measures local-scope lookup vs global baseline.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

def _process(v: int, offset: int) -> int:
    local_a = v + offset
    local_b = local_a * 3
    return local_b & 0xFFFF

acc = 0
for i in range(ITERS):
    acc ^= _process(_inputs[i % _n], i & 0xFF)

# Stdout sink — byte-equal across runtimes.
print(f"scope_lookup: {ITERS}")
