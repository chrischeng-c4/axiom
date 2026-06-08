"""Hot-loop bench for language bitwise: integer bitwise ops.

Domain: language
Feature: bitwise
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop applying AND/OR/XOR/shift operations on
int inputs — monomorphic int regime, measures bitwise op throughput.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    acc ^= ((v << 3) | (v >> 2)) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"bitwise_int: {ITERS}")
