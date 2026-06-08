"""Behavior contract for builtins.max.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: max of list of ints
result = max([3, 1, 4, 1, 5, 9, 2, 6])
assert result == 9, f"max([3,1,4,1,5,9,2,6]) = {result!r}, expected 9"

# Rule 2: max of two positional args
result = max(10, 3)
assert result == 10, f"max(10, 3) = {result!r}, expected 10"

# Rule 3: max of multiple positional args
result = max(5, 2, 8, 1, 9)
assert result == 9, f"max(5,2,8,1,9) = {result!r}, expected 9"

# Rule 4: max of strings (lexicographic)
result = max("banana", "apple", "cherry")
assert result == "cherry", \
    f"max('banana','apple','cherry') = {result!r}, expected 'cherry'"

# Rule 5: max with key function
result = max([3, 1, 4, 1, 5], key=lambda x: -x)
assert result == 1, f"max([3,1,4,1,5], key=lambda x:-x) = {result!r}, expected 1"

# Rule 6: max of empty list raises ValueError
_raised = False
try:
    max([])
except ValueError:
    _raised = True
assert _raised, "max([]) did not raise ValueError"

# Rule 7: max with default kwarg on empty
result = max([], default=0)
assert result == 0, f"max([], default=0) = {result!r}, expected 0"

# Rule 8: max of single-element list
result = max([42])
assert result == 42, f"max([42]) = {result!r}, expected 42"

# Rule 9: max of negative ints
result = max([-3, -1, -4, -1, -5])
assert result == -1, f"max([-3,-1,-4,-1,-5]) = {result!r}, expected -1"

# Rule 10: TypeError for non-iterable single arg
_raised = False
try:
    max(42)
except TypeError:
    _raised = True
assert _raised, "max(42) did not raise TypeError"

# Rule 11: max of tuple
result = max((5, 3, 8, 1))
assert result == 8, f"max((5,3,8,1)) = {result!r}, expected 8"

# Rule 12: max with key=abs
result = max([-5, 3, -1, 4], key=abs)
assert result == -5, f"max([-5,3,-1,4], key=abs) = {result!r}, expected -5"

# Rule 13: return type matches input element type
result = max([1, 2, 3])
assert type(result) is int, \
    f"type(max([1,2,3])) = {type(result).__name__!r}, expected 'int'"

result = max([1.5, 2.5, 0.5])
assert type(result) is float, \
    f"type(max([1.5,2.5,0.5])) = {type(result).__name__!r}, expected 'float'"

print("behavior OK")
