# Operational AssertionPass seed for itertools surfaces beyond
# test_itertools_ops / test_itertools_combinatoric_ops.
# Surface: islice with 1-arg (stop), 2-arg (start/stop), and 3-arg
# (start/stop/step) forms; takewhile takes from the front until the
# predicate is False; dropwhile skips from the front until the
# predicate is False, then yields everything; accumulate yields a
# running total (default add, custom binary func also works); repeat
# with a `times=` limit; starmap unpacks each tuple as the callable's
# args; tee produces independent iterators over the same source;
# pairwise produces overlapping adjacent pairs; compress keeps items
# where the parallel selector is truthy; filterfalse is the inverse
# of filter.
import itertools
_ledger: list[int] = []

src = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

# islice with a single arg takes the first n items
assert list(itertools.islice(src, 3)) == [1, 2, 3]; _ledger.append(1)
# 2-arg islice (start, stop) — half-open
assert list(itertools.islice(src, 2, 5)) == [3, 4, 5]; _ledger.append(1)
# 3-arg islice (start, stop, step)
assert list(itertools.islice(src, 0, 10, 2)) == [1, 3, 5, 7, 9]; _ledger.append(1)

# takewhile yields from the front until the predicate returns False;
# nothing after the first failure is emitted
assert list(itertools.takewhile(lambda x: x < 5, [1, 2, 3, 4, 5, 6, 1])) == [1, 2, 3, 4]; _ledger.append(1)

# dropwhile skips items while the predicate is True, then yields the
# rest unchanged
assert list(itertools.dropwhile(lambda x: x < 5, [1, 2, 3, 4, 5, 6, 1])) == [5, 6, 1]; _ledger.append(1)

# accumulate produces the running sum by default
assert list(itertools.accumulate([1, 2, 3, 4])) == [1, 3, 6, 10]; _ledger.append(1)
# accumulate with a custom binary function (running product)
assert list(itertools.accumulate([1, 2, 3, 4], lambda a, b: a * b)) == [1, 2, 6, 24]; _ledger.append(1)

# repeat with a times limit produces exactly that many copies
assert list(itertools.repeat("x", 3)) == ["x", "x", "x"]; _ledger.append(1)
# repeat with times=0 produces an empty iterable
assert list(itertools.repeat("x", 0)) == []; _ledger.append(1)

# starmap unpacks each tuple as the callable's positional args
assert list(itertools.starmap(lambda a, b: a + b, [(1, 2), (3, 4), (5, 6)])) == [3, 7, 11]; _ledger.append(1)

# tee returns n independent iterators over the same source
ta, tb = itertools.tee([1, 2, 3])
assert list(ta) == [1, 2, 3]; _ledger.append(1)
# The second iterator yields the same values independently
assert list(tb) == [1, 2, 3]; _ledger.append(1)

# pairwise yields overlapping adjacent pairs
assert list(itertools.pairwise([1, 2, 3, 4])) == [(1, 2), (2, 3), (3, 4)]; _ledger.append(1)
# pairwise on a 1-element iterable yields no pairs
assert list(itertools.pairwise([1])) == []; _ledger.append(1)

# compress keeps items where the parallel selector is truthy
assert list(itertools.compress(["a", "b", "c", "d"], [1, 0, 1, 0])) == ["a", "c"]; _ledger.append(1)
# compress with all-falsy selector yields nothing
assert list(itertools.compress(["a", "b", "c"], [0, 0, 0])) == []; _ledger.append(1)

# filterfalse is the inverse of filter — keeps items where the
# predicate returns False
assert list(itertools.filterfalse(lambda x: x < 3, [1, 2, 3, 4, 5])) == [3, 4, 5]; _ledger.append(1)
# filterfalse with always-True predicate keeps nothing
assert list(itertools.filterfalse(lambda x: True, [1, 2, 3])) == []; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_itertools_chains_ops {sum(_ledger)} asserts")
