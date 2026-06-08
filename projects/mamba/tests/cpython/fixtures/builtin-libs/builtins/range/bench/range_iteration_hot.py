"""Hot-loop bench for builtins.range: integer range iteration.

Domain: builtins
Feature: range
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop iterating a range object directly —
the most common Python loop pattern, fully monomorphic on int.
"""
# tier: compute


ITERS = 100_000

acc = 0
for _ in range(ITERS):
    for v in range(100):
        acc ^= v

# Stdout sink — byte-equal across runtimes.
print(f"range_iteration: {ITERS}")
