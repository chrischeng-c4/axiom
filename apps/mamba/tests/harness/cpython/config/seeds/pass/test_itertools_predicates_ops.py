# Operational AssertionPass seed for itertools surfaces beyond the
# chains/combinatoric trio (test_itertools_ops / test_itertools_chains_ops /
# test_itertools_combinatoric_ops). Focus: predicate-driven and
# accumulating iterators. Surface: accumulate (running cumulative sum
# on a list of ints — empty list, single-element, and several
# elements); islice (n-element prefix, [start, stop) slice, and
# strided [start, stop, step] slice on a range source); takewhile
# (returns the leading run that satisfies the predicate, stops at the
# first violator); dropwhile (drops the leading run, returns the rest
# INCLUDING any later predicate-satisfying elements); filterfalse
# (the inverse of filter — keeps elements where the predicate is
# false); starmap (unpacks tuple arguments to a function); chain on
# two and on multiple iterables; repeat with an explicit count;
# zip_longest with and without fillvalue (default fill is None).
import itertools
_ledger: list[int] = []

# accumulate — running cumulative sum
assert list(itertools.accumulate([1, 2, 3, 4])) == [1, 3, 6, 10]; _ledger.append(1)
assert list(itertools.accumulate([5])) == [5]; _ledger.append(1)
assert list(itertools.accumulate([])) == []; _ledger.append(1)
assert list(itertools.accumulate([10, 20, 30, 40, 50])) == [10, 30, 60, 100, 150]; _ledger.append(1)

# islice — n-element prefix
assert list(itertools.islice(range(10), 5)) == [0, 1, 2, 3, 4]; _ledger.append(1)
# [start, stop) slice
assert list(itertools.islice(range(10), 2, 7)) == [2, 3, 4, 5, 6]; _ledger.append(1)
# [start, stop, step] strided slice
assert list(itertools.islice(range(10), 0, 10, 2)) == [0, 2, 4, 6, 8]; _ledger.append(1)
# Empty source
assert list(itertools.islice([], 5)) == []; _ledger.append(1)
# n=0 returns empty
assert list(itertools.islice(range(10), 0)) == []; _ledger.append(1)

# takewhile — returns the leading run; stops at first violator
assert list(itertools.takewhile(lambda x: x < 3, [1, 2, 3, 4])) == [1, 2]; _ledger.append(1)
# All match → return everything
assert list(itertools.takewhile(lambda x: x < 100, [1, 2, 3])) == [1, 2, 3]; _ledger.append(1)
# None match → return empty
assert list(itertools.takewhile(lambda x: x < 0, [1, 2, 3])) == []; _ledger.append(1)

# dropwhile — drops the leading match-run; returns the rest (including
# later predicate-matching elements!)
assert list(itertools.dropwhile(lambda x: x < 3, [1, 2, 3, 4, 1])) == [3, 4, 1]; _ledger.append(1)
# All match → drop everything
assert list(itertools.dropwhile(lambda x: x < 100, [1, 2, 3])) == []; _ledger.append(1)
# None match → drop nothing
assert list(itertools.dropwhile(lambda x: x < 0, [1, 2, 3])) == [1, 2, 3]; _ledger.append(1)

# filterfalse — inverse of filter (keeps where predicate is FALSE)
assert list(itertools.filterfalse(lambda x: x % 2 == 0, [1, 2, 3, 4, 5])) == [1, 3, 5]; _ledger.append(1)
assert list(itertools.filterfalse(lambda x: x > 10, [1, 2, 3, 20, 30])) == [1, 2, 3]; _ledger.append(1)
# Empty source
assert list(itertools.filterfalse(lambda x: True, [])) == []; _ledger.append(1)

# starmap — unpack each tuple as args to the function
assert list(itertools.starmap(lambda a, b: a + b, [(1, 2), (3, 4)])) == [3, 7]; _ledger.append(1)
assert list(itertools.starmap(lambda a, b: a * b, [(2, 3), (4, 5), (6, 7)])) == [6, 20, 42]; _ledger.append(1)

# chain — two iterables
assert list(itertools.chain([1, 2], [3, 4])) == [1, 2, 3, 4]; _ledger.append(1)
# chain — many iterables
assert list(itertools.chain([1], [2], [3], [4])) == [1, 2, 3, 4]; _ledger.append(1)
# chain with an empty iterable in the middle degenerates correctly
assert list(itertools.chain([1, 2], [], [3, 4])) == [1, 2, 3, 4]; _ledger.append(1)

# repeat — fixed-count repetition
assert list(itertools.repeat("x", 3)) == ["x", "x", "x"]; _ledger.append(1)
assert list(itertools.repeat(42, 5)) == [42, 42, 42, 42, 42]; _ledger.append(1)
assert list(itertools.repeat(0, 0)) == []; _ledger.append(1)

# zip_longest — default fill is None
assert list(itertools.zip_longest([1, 2], ["a", "b", "c"])) == [(1, "a"), (2, "b"), (None, "c")]; _ledger.append(1)
# Explicit fillvalue
assert list(itertools.zip_longest([1, 2], ["a", "b", "c"], fillvalue=0)) == [(1, "a"), (2, "b"), (0, "c")]; _ledger.append(1)
# Equal-length is just zip
assert list(itertools.zip_longest([1, 2], ["a", "b"])) == [(1, "a"), (2, "b")]; _ledger.append(1)

# Pipeline of multiple iterators — chain → islice → list
chained = itertools.chain([1, 2, 3], [4, 5, 6])
assert list(itertools.islice(chained, 2, 5)) == [3, 4, 5]; _ledger.append(1)

# Pipeline — filterfalse → list (drops evens, keeps odds)
odds = itertools.filterfalse(lambda x: x % 2 == 0, range(8))
assert list(odds) == [1, 3, 5, 7]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_itertools_predicates_ops {sum(_ledger)} asserts")
