"""Hot-loop bench for builtins.len: list length.

Domain: builtins
Feature: len
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling len() on a pre-allocated list
of fixed size — the canonical monomorphic case where mamba can
specialize the call site on list and lower to a native field read.
"""
# tier: compute


ITERS = 500_000

# Build input outside the loop — one list, always the same type.
_lst = list(range(100))

acc = 0
for i in range(ITERS):
    acc ^= len(_lst)

# Stdout sink — byte-equal across runtimes.
print(f"len_list: {ITERS}")
