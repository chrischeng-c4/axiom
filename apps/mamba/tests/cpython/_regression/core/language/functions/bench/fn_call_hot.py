"""Hot-loop bench for language functions: function call overhead.

Domain: language
Feature: functions
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling a simple int→int function —
monomorphic on int inputs, measures bare call overhead.
"""
# tier: compute


ITERS = 500_000

def _double(x: int) -> int:
    return x * 2

_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    acc ^= _double(_inputs[i % _n])

# Stdout sink — byte-equal across runtimes.
print(f"fn_call: {ITERS}")
