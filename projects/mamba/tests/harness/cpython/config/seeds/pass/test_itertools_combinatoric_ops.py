# Operational AssertionPass seed for `itertools` combinatoric +
# infinite-stream operators.
# Surface: chain, repeat, islice, accumulate, combinations,
# permutations, product. Each iterator is materialised with list() so
# the assertion compares against a concrete sequence.
import itertools
_ledger: list[int] = []
# chain flattens two iterables
assert list(itertools.chain([1, 2], [3, 4])) == [1, 2, 3, 4]; _ledger.append(1)
# repeat with explicit count yields exactly that many items
assert list(itertools.repeat("x", 3)) == ["x", "x", "x"]; _ledger.append(1)
# islice slices an iterator using [start, stop) semantics
assert list(itertools.islice(range(10), 3, 7)) == [3, 4, 5, 6]; _ledger.append(1)
# accumulate computes running sums by default
assert list(itertools.accumulate([1, 2, 3, 4])) == [1, 3, 6, 10]; _ledger.append(1)
# combinations: order-independent k-subsets, lexicographically
assert list(itertools.combinations([1, 2, 3], 2)) == [(1, 2), (1, 3), (2, 3)]; _ledger.append(1)
# permutations: ordered k-tuples
assert list(itertools.permutations([1, 2, 3], 2)) == [(1, 2), (1, 3), (2, 1), (2, 3), (3, 1), (3, 2)]; _ledger.append(1)
# product: cartesian product across input iterables
assert list(itertools.product([1, 2], [3, 4])) == [(1, 3), (1, 4), (2, 3), (2, 4)]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_itertools_combinatoric_ops {sum(_ledger)} asserts")
