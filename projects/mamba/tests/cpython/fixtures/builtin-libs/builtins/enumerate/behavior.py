"""Behavior contract for builtins.enumerate.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: enumerate with default start=0
result = list(enumerate(["a", "b", "c"]))
assert result == [(0, "a"), (1, "b"), (2, "c")], \
    f"enumerate(['a','b','c']) = {result!r}"

# Rule 2: enumerate with explicit start
result = list(enumerate(["a", "b", "c"], start=5))
assert result == [(5, "a"), (6, "b"), (7, "c")], \
    f"enumerate(['a','b','c'], start=5) = {result!r}"

# Rule 3: enumerate of empty iterable
result = list(enumerate([]))
assert result == [], f"enumerate([]) = {result!r}, expected []"

# Rule 4: enumerate returns iterator (has __next__)
_it = enumerate(["a"])
assert hasattr(_it, "__next__"), \
    f"enumerate result has no __next__: type={type(_it).__name__}"

# Rule 5: each element is a tuple
result = list(enumerate([10, 20]))
assert all(type(t) is tuple for t in result), \
    f"enumerate items not tuples: {[type(t).__name__ for t in result]}"

# Rule 6: tuple unpacking in for-loop
_pairs = []
for i, v in enumerate([10, 20, 30]):
    _pairs.append((i, v))
assert _pairs == [(0, 10), (1, 20), (2, 30)], \
    f"enumerate for-loop unpacking = {_pairs!r}"

# Rule 7: enumerate of string
result = list(enumerate("abc"))
assert result == [(0, "a"), (1, "b"), (2, "c")], \
    f"enumerate('abc') = {result!r}"

# Rule 8: enumerate of range
result = list(enumerate(range(3)))
assert result == [(0, 0), (1, 1), (2, 2)], \
    f"enumerate(range(3)) = {result!r}"

# Rule 9: negative start
result = list(enumerate(["x", "y"], start=-1))
assert result == [(-1, "x"), (0, "y")], \
    f"enumerate start=-1 = {result!r}"

# Rule 10: index part is always int
result = list(enumerate([1, 2, 3]))
assert all(type(idx) is int for idx, _ in result), \
    f"enumerate index types: {[type(idx).__name__ for idx, _ in result]}"

print("behavior OK")
