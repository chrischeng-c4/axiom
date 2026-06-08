"""Behavior contract for builtins.min.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: min of list of ints
result = min([3, 1, 4, 1, 5, 9, 2, 6])
assert result == 1, f"min([3,1,4,1,5,9,2,6]) = {result!r}, expected 1"

# Rule 2: min of two positional args
result = min(10, 3)
assert result == 3, f"min(10, 3) = {result!r}, expected 3"

# Rule 3: min of multiple positional args
result = min(5, 2, 8, 1, 9)
assert result == 1, f"min(5,2,8,1,9) = {result!r}, expected 1"

# Rule 4: min of strings (lexicographic)
result = min("banana", "apple", "cherry")
assert result == "apple", f"min('banana','apple','cherry') = {result!r}, expected 'apple'"

# Rule 5: min with key function
result = min([3, 1, 4, 1, 5], key=lambda x: -x)
assert result == 5, f"min([3,1,4,1,5], key=lambda x:-x) = {result!r}, expected 5"

# Rule 6: min with default on empty (no default raises ValueError)
_raised = False
try:
    min([])
except ValueError:
    _raised = True
assert _raised, "min([]) did not raise ValueError"

# Rule 7: min with default kwarg on empty
result = min([], default=99)
assert result == 99, f"min([], default=99) = {result!r}, expected 99"

# Rule 8: min returns the actual element (not a copy)
_lst = [3, 1, 4]
result = min(_lst)
assert result == 1, f"min([3,1,4]) = {result!r}, expected 1"

# Rule 9: min of single-element list
result = min([42])
assert result == 42, f"min([42]) = {result!r}, expected 42"

# Rule 10: min of negative ints
result = min([-3, -1, -4, -1, -5])
assert result == -5, f"min([-3,-1,-4,-1,-5]) = {result!r}, expected -5"

# Rule 11: min of single positional arg must be iterable
_raised = False
try:
    min(42)
except TypeError:
    _raised = True
assert _raised, "min(42) did not raise TypeError"

# Rule 12: min of tuple
result = min((5, 3, 8, 1))
assert result == 1, f"min((5,3,8,1)) = {result!r}, expected 1"

# Rule 13: min with key=abs
result = min([-5, 3, -1, 4], key=abs)
assert result == -1, f"min([-5,3,-1,4], key=abs) = {result!r}, expected -1"

# Rule 14: return type matches input element type
result = min([1, 2, 3])
assert type(result) is int, \
    f"type(min([1,2,3])) = {type(result).__name__!r}, expected 'int'"

result = min([1.5, 2.5, 0.5])
assert type(result) is float, \
    f"type(min([1.5,2.5,0.5])) = {type(result).__name__!r}, expected 'float'"

print("behavior OK")
