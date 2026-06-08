"""Hot-loop bench for language generators: generator iteration.

Domain: language
Feature: generators
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop creating and exhausting a small generator —
monomorphic on int yields, exercises generator frame overhead.
"""
# tier: compute


ITERS = 200_000

def _range_gen(n: int):
    for i in range(n):
        yield i

acc = 0
for _outer in range(ITERS):
    for v in _range_gen(5):
        acc ^= v

# Stdout sink — byte-equal across runtimes.
print(f"gen_iter: {ITERS}")
