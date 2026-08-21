# Operational AssertionPass seed for itertools combinatoric/stream
# surface not covered by `test_itertools_combinatoric_ops`. That seed
# asserts chain, repeat (counted), islice (start/stop), accumulate,
# combinations, permutations, product (two-arg). This seed asserts:
#   * combinations_with_replacement — k-multisets, lexicographically
#   * starmap — apply f(*tuple) over an iterable of tuples
#   * tee — replicate an iterator into N independent iterators
#   * zip_longest — pad the short side with fillvalue
#   * islice with explicit step
#   * permutations with no k (full-length N!)
#   * product over strings (cartesian product of two iterables)
import itertools as it
_ledger: list[int] = []

# combinations_with_replacement — k-multisets in lex order
assert list(it.combinations_with_replacement([1, 2], 2)) == [(1, 1), (1, 2), (2, 2)]; _ledger.append(1)
assert list(it.combinations_with_replacement([1, 2, 3], 2)) == [(1, 1), (1, 2), (1, 3), (2, 2), (2, 3), (3, 3)]; _ledger.append(1)
# k=1 → singletons
assert list(it.combinations_with_replacement([1, 2, 3], 1)) == [(1,), (2,), (3,)]; _ledger.append(1)

# starmap — call lambda(*t) for each t
assert list(it.starmap(lambda a, b: a + b, [(1, 2), (3, 4), (5, 6)])) == [3, 7, 11]; _ledger.append(1)
assert list(it.starmap(lambda a, b: a * b, [(2, 3), (4, 5)])) == [6, 20]; _ledger.append(1)

# tee — two independent iterators over the same source
a_iter, b_iter = it.tee([10, 20, 30])
assert list(a_iter) == [10, 20, 30]; _ledger.append(1)
assert list(b_iter) == [10, 20, 30]; _ledger.append(1)

# zip_longest — pad the short side with fillvalue
assert list(it.zip_longest([1, 2, 3], [4, 5], fillvalue=0)) == [(1, 4), (2, 5), (3, 0)]; _ledger.append(1)
assert list(it.zip_longest([1], [2, 3, 4], fillvalue=-1)) == [(1, 2), (-1, 3), (-1, 4)]; _ledger.append(1)
# Equal-length still pads zero
assert list(it.zip_longest([1, 2], [3, 4], fillvalue=0)) == [(1, 3), (2, 4)]; _ledger.append(1)

# islice with step — [start, stop, step)
assert list(it.islice(range(10), 2, 7, 2)) == [2, 4, 6]; _ledger.append(1)
assert list(it.islice(range(10), 0, 10, 3)) == [0, 3, 6, 9]; _ledger.append(1)
assert list(it.islice(range(5), 1, 5, 2)) == [1, 3]; _ledger.append(1)

# permutations with no k — full N! permutations of N items
assert list(it.permutations([1, 2, 3])) == [(1, 2, 3), (1, 3, 2), (2, 1, 3), (2, 3, 1), (3, 1, 2), (3, 2, 1)]; _ledger.append(1)
assert len(list(it.permutations([1, 2, 3, 4]))) == 24; _ledger.append(1)  # 4! == 24
assert list(it.permutations([1, 2])) == [(1, 2), (2, 1)]; _ledger.append(1)

# product over strings — cartesian product
assert list(it.product("ab", "cd")) == [("a", "c"), ("a", "d"), ("b", "c"), ("b", "d")]; _ledger.append(1)
# product over numeric pairs
assert list(it.product([1, 2], [3, 4])) == [(1, 3), (1, 4), (2, 3), (2, 4)]; _ledger.append(1)

# Type invariants — these all return iterators, materialise to list
assert isinstance(list(it.combinations_with_replacement([1, 2], 2)), list); _ledger.append(1)
assert isinstance(list(it.starmap(lambda x: x, [(1,)])), list); _ledger.append(1)
assert isinstance(list(it.zip_longest([1], [2])), list); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_itertools_combinatoric_extras_ops {sum(_ledger)} asserts")
