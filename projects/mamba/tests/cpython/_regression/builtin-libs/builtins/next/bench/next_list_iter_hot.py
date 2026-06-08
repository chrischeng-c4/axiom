"""Hot-loop bench for builtins.next: manual iterator advancement.

Domain: builtins
Feature: next
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop manually advancing an iterator with next()
calls — monomorphic on list_iterator[int].
"""
# tier: compute


ITERS = 200_000

_lst = list(range(10))

acc = 0
for i in range(ITERS):
    _it = iter(_lst)
    acc ^= next(_it)
    acc ^= next(_it)
    acc ^= next(_it)

# Stdout sink — byte-equal across runtimes.
print(f"next_list_iter: {ITERS}")
