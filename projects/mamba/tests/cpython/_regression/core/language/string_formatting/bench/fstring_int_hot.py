"""Hot-loop bench for language string formatting: f-string with int.

Domain: language
Feature: string_formatting
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop building f-strings from integer values —
monomorphic int regime, measures f-string construction overhead.
"""
# tier: compute


ITERS = 500_000

_inputs = list(range(100))
_n = len(_inputs)

acc = 0
for i in range(ITERS):
    v = _inputs[i % _n]
    _s = f"{v:04d}"
    acc ^= len(_s) & 0xFFFF

# Stdout sink — byte-equal across runtimes.
print(f"fstring_int: {ITERS}")
