"""Hot-loop bench for builtins.all: bool list check.

Domain: builtins
Feature: all
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop checking all() on a pre-built bool list —
monomorphic on list[bool], all-true path (no short-circuit).
"""
# tier: compute


ITERS = 200_000

# All-true list — tests the no-short-circuit path (worst case for all).
_lst = [True] * 100

acc = 0
for i in range(ITERS):
    acc ^= all(_lst)

# Stdout sink — byte-equal across runtimes.
print(f"all_bool_list: {ITERS}")
