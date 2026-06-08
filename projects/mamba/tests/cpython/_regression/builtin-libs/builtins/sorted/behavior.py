"""Behavior contract for builtins.sorted.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: sorted of list of ints (ascending default)
result = sorted([3, 1, 4, 1, 5, 9, 2, 6])
assert result == [1, 1, 2, 3, 4, 5, 6, 9], \
    f"sorted([3,1,4,1,5,9,2,6]) = {result!r}"

# Rule 2: sorted always returns a new list
_orig = [3, 1, 2]
result = sorted(_orig)
assert result is not _orig, "sorted() returned the same object"
assert type(result) is list, \
    f"type(sorted(...)) = {type(result).__name__!r}, expected 'list'"

# Rule 3: sorted of empty iterable
result = sorted([])
assert result == [], f"sorted([]) = {result!r}, expected []"

# Rule 4: sorted with reverse=True
result = sorted([3, 1, 4, 1, 5], reverse=True)
assert result == [5, 4, 3, 1, 1], \
    f"sorted([3,1,4,1,5], reverse=True) = {result!r}"

# Rule 5: sorted with key function
result = sorted([3, -1, 4, -2, 5], key=abs)
assert result == [-1, -2, 3, 4, 5], \
    f"sorted([3,-1,4,-2,5], key=abs) = {result!r}"

# Rule 6: sorted of strings (lexicographic)
result = sorted(["banana", "apple", "cherry"])
assert result == ["apple", "banana", "cherry"], \
    f"sorted(['banana','apple','cherry']) = {result!r}"

# Rule 7: sorted of tuple returns list
result = sorted((5, 3, 8, 1))
assert result == [1, 3, 5, 8], f"sorted((5,3,8,1)) = {result!r}"
assert type(result) is list, \
    f"type(sorted(tuple)) = {type(result).__name__!r}, expected 'list'"

# Rule 8: sorted of range returns list
result = sorted(range(5, 0, -1))
assert result == [1, 2, 3, 4, 5], f"sorted(range(5,0,-1)) = {result!r}"

# Rule 9: sorted is stable (equal elements preserve order)
_data = [(1, "b"), (1, "a"), (2, "c")]
result = sorted(_data, key=lambda x: x[0])
assert result == [(1, "b"), (1, "a"), (2, "c")], \
    f"stable sort = {result!r}"

# Rule 10: TypeError for non-comparable elements
_raised = False
try:
    sorted([1, "a", 2])
except TypeError:
    _raised = True
assert _raised, "sorted([1,'a',2]) did not raise TypeError"

# Rule 11: sorted with key=str.lower for case-insensitive sort
result = sorted(["Banana", "apple", "Cherry"], key=str.lower)
assert result == ["apple", "Banana", "Cherry"], \
    f"sorted case-insensitive = {result!r}"

# Rule 12: sorted of single-element
result = sorted([42])
assert result == [42], f"sorted([42]) = {result!r}"

print("behavior OK")
