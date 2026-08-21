"""Hot-loop bench for builtins.zip: two-list zip iteration.

Domain: builtins
Feature: zip
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop zipping two same-type lists and summing
the paired products — canonical monomorphic use of zip over int lists.
"""
# tier: compute


ITERS = 50_000

# Build inputs outside the loop.
_xs = list(range(100))
_ys = list(range(100, 200))

acc = 0
for i in range(ITERS):
    for x, y in zip(_xs, _ys):
        acc ^= x + y

# Stdout sink — byte-equal across runtimes.
print(f"zip_two_lists: {ITERS}")
