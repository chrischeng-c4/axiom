# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/sort_methods: behavior asserts (CPython 3.12 oracle)."""

# Core sort behaviors that match CPython expectations.
assert sorted([3, 1, 2]) == [1, 2, 3]
assert sorted([3, 1, 2], reverse=True) == [3, 2, 1]

# sorted() returns a new list; list.sort() mutates in place and returns None.
src = [3, 1, 2]
out = sorted(src)
assert src == [3, 1, 2] and out == [1, 2, 3]
assert src.sort() is None and src == [1, 2, 3]
print("new_vs_inplace: ok")

# reverse=True on a shuffled range yields the exact descending range.
desc = sorted(range(100), reverse=True)
assert desc == list(range(99, -1, -1))
print("reverse_range: ok")

# Tuples compare lexicographically: a tuple whose first elements are all
# equal (here None == None) is ordered by its second element. CPython's
# optimized tuple compare short-circuits the equal head and never tries to
# order None against None with '<'.
none_pairs = sorted([(None, 2), (None, 1)])
assert none_pairs == [(None, 1), (None, 2)]
print("none_in_tuples:", none_pairs)

# Lexicographic ordering of comparable tuples.
assert sorted([(1, "b"), (1, "a"), (0, "z")]) == [(0, "z"), (1, "a"), (1, "b")]

# Booleans sort as 0/1 and interleave with ints.
assert sorted([True, 0, False, 1, 2]) == [0, False, True, 1, 2]
print("bool_as_int: ok")

# sorted() accepts any iterable, always returning a list.
assert sorted("dcba") == ["a", "b", "c", "d"]
assert sorted({3, 1, 2}) == [1, 2, 3]
assert sorted((2, 3, 1)) == [1, 2, 3]
assert sorted({"b": 1, "a": 2}) == ["a", "b"]
print("iterables: ok")

print("behavior OK")
