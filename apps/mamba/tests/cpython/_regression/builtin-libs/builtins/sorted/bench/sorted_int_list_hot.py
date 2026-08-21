"""Hot-loop bench for builtins.sorted: integer list sort.

Domain: builtins
Feature: sorted
Tier: app

# type-regime: monomorphic

End-user scenario: repeated sorted() over a fixed-size shuffled list
of integers — realistic sort workload, always the same element type.
"""
# tier: app


ITERS = 5_000

# Build input outside the loop — shuffled list of int, always same type.
# Use a deterministic shuffle so output is byte-equal.
_base = list(range(200))
# Deterministic knuth shuffle with fixed seed
_state = 42
_shuffled = _base[:]
for _i in range(len(_shuffled) - 1, 0, -1):
    _state = (_state * 6364136223846793005 + 1442695040888963407) & 0xFFFFFFFFFFFFFFFF
    _j = _state % (_i + 1)
    _shuffled[_i], _shuffled[_j] = _shuffled[_j], _shuffled[_i]

acc = 0
for i in range(ITERS):
    _out = sorted(_shuffled)
    acc ^= _out[0]

# Stdout sink — byte-equal across runtimes.
print(f"sorted_int_list: {ITERS}")
