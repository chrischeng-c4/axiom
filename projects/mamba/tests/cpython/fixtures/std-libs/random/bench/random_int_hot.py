"""Hot-loop bench for stdlib random: randint in tight loop.

Domain: stdlib
Feature: random
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop generating random integers with seeded RNG —
monomorphic int regime, measures random.randint throughput.
"""
# tier: compute

import random

random.seed(42)

ITERS = 500_000

acc = 0
for i in range(ITERS):
    acc ^= random.randint(0, 255)

# Stdout sink — byte-equal across runtimes.
print(f"random_int: {ITERS}")
