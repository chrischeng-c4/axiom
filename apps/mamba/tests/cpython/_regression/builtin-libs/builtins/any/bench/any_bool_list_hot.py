"""Hot-loop bench for builtins.any: bool list check.

Domain: builtins
Feature: any
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop checking any() on a pre-built all-false
bool list — monomorphic on list[bool], no-match path (no short-circuit,
full scan).
"""
# tier: compute


ITERS = 200_000

# All-false list — tests the full-scan path (worst case for any).
_lst = [False] * 100

acc = 0
for i in range(ITERS):
    acc ^= any(_lst)

# Stdout sink — byte-equal across runtimes.
print(f"any_bool_list: {ITERS}")
