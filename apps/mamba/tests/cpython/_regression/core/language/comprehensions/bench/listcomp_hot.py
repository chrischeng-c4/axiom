"""Hot-loop bench for language comprehensions: list comprehension.

Domain: language
Feature: comprehensions
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop building a list comprehension from a
fixed 100-element input — monomorphic on int transforms.
"""
# tier: compute


ITERS = 100_000

_src = list(range(100))

acc = 0
for _ in range(ITERS):
    result = [x * x for x in _src]
    acc ^= result[-1]

# Stdout sink — byte-equal across runtimes.
print(f"listcomp: {ITERS}")
