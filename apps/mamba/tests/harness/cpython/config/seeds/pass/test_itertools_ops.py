# Operational AssertionPass seed for `itertools` working surface.
# Surface: chain, accumulate, combinations (order-preserving),
# permutations (all orderings), product (cartesian).
# Companion to stub/test_itertools.py — vendored unittest seed.
from itertools import chain, accumulate, combinations, permutations, product
_ledger: list[int] = []
assert list(chain([1, 2], [3, 4], [5])) == [1, 2, 3, 4, 5]; _ledger.append(1)
assert list(chain([], [1], [])) == [1]; _ledger.append(1)
assert list(accumulate([1, 2, 3, 4, 5])) == [1, 3, 6, 10, 15]; _ledger.append(1)
assert list(accumulate([1, 2, 3], lambda a, b: a * b)) == [1, 2, 6]; _ledger.append(1)
assert list(combinations([1, 2, 3], 2)) == [(1, 2), (1, 3), (2, 3)]; _ledger.append(1)
assert list(combinations([1, 2, 3, 4], 3)) == [(1, 2, 3), (1, 2, 4), (1, 3, 4), (2, 3, 4)]; _ledger.append(1)
perms = list(permutations([1, 2], 2))
assert sorted(perms) == [(1, 2), (2, 1)]; _ledger.append(1)
prods = list(product([1, 2], ["a", "b"]))
assert prods == [(1, "a"), (1, "b"), (2, "a"), (2, "b")]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_itertools_ops {sum(_ledger)} asserts")
