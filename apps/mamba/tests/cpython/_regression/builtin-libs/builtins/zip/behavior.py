"""Behavior contract for builtins.zip.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: zip two equal-length lists
result = list(zip([1, 2, 3], [4, 5, 6]))
assert result == [(1, 4), (2, 5), (3, 6)], \
    f"zip two lists = {result!r}"

# Rule 2: zip truncates to shortest
result = list(zip([1, 2, 3], [4, 5]))
assert result == [(1, 4), (2, 5)], \
    f"zip truncates to shortest = {result!r}"

# Rule 3: zip with no args returns empty
result = list(zip())
assert result == [], f"list(zip()) = {result!r}, expected []"

# Rule 4: zip three iterables
result = list(zip([1, 2], [3, 4], [5, 6]))
assert result == [(1, 3, 5), (2, 4, 6)], \
    f"zip three lists = {result!r}"

# Rule 5: zip with empty iterable yields nothing
result = list(zip([], [1, 2, 3]))
assert result == [], f"zip([], [1,2,3]) = {result!r}, expected []"

# Rule 6: zip returns iterator, not list
_it = zip([1, 2], [3, 4])
assert hasattr(_it, "__next__"), \
    f"zip() result has no __next__: type={type(_it).__name__}"

# Rule 7: zip tuples are tuples (not lists)
result = list(zip([1, 2], [3, 4]))
assert all(type(t) is tuple for t in result), \
    f"zip tuples are not tuples: {[type(t).__name__ for t in result]}"

# Rule 8: zip with range
result = list(zip(range(3), range(10, 13)))
assert result == [(0, 10), (1, 11), (2, 12)], \
    f"zip(range(3), range(10,13)) = {result!r}"

# Rule 9: zip with strict=True raises ValueError when lengths differ (Python 3.10+)
_raised = False
try:
    list(zip([1, 2, 3], [1, 2], strict=True))
except ValueError:
    _raised = True
assert _raised, "zip(..., strict=True) with unequal lengths did not raise ValueError"

# Rule 10: zip with strict=True succeeds when same length
result = list(zip([1, 2], [3, 4], strict=True))
assert result == [(1, 3), (2, 4)], \
    f"zip strict same-length = {result!r}"

print("behavior OK")
