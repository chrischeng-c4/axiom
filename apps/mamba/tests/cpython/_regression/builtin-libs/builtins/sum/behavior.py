"""Behavior contract for builtins.sum.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: sum of list of ints
result = sum([1, 2, 3, 4, 5])
assert result == 15, f"sum([1,2,3,4,5]) = {result!r}, expected 15"

# Rule 2: sum of empty iterable (default start=0)
result = sum([])
assert result == 0, f"sum([]) = {result!r}, expected 0"

# Rule 3: sum with explicit start value
result = sum([1, 2, 3], 10)
assert result == 16, f"sum([1,2,3], 10) = {result!r}, expected 16"

# Rule 4: sum of tuple
result = sum((10, 20, 30))
assert result == 60, f"sum((10,20,30)) = {result!r}, expected 60"

# Rule 5: sum returns int for int inputs
result = sum([1, 2, 3])
assert type(result) is int, \
    f"type(sum([1,2,3])) = {type(result).__name__!r}, expected 'int'"

# Rule 6: sum of floats returns float
result = sum([1.0, 2.0, 3.0])
assert result == 6.0, f"sum([1.0,2.0,3.0]) = {result!r}, expected 6.0"
assert type(result) is float, \
    f"type(sum([1.0,2.0,3.0])) = {type(result).__name__!r}, expected 'float'"

# Rule 7: sum with float start upgrades result to float
result = sum([1, 2, 3], 0.0)
assert result == 6.0, f"sum([1,2,3], 0.0) = {result!r}, expected 6.0"

# Rule 8: sum of single-element list
result = sum([42])
assert result == 42, f"sum([42]) = {result!r}, expected 42"

# Rule 9: sum of negative ints
result = sum([-1, -2, -3])
assert result == -6, f"sum([-1,-2,-3]) = {result!r}, expected -6"

# Rule 10: sum of generator expression
result = sum(i * i for i in range(5))
assert result == 30, f"sum(i*i for i in range(5)) = {result!r}, expected 30"

# Rule 11: sum of range
result = sum(range(1, 6))
assert result == 15, f"sum(range(1,6)) = {result!r}, expected 15"

# Rule 12: TypeError for string start (sum doesn't accept str start)
_raised = False
try:
    sum([1, 2], "x")
except TypeError:
    _raised = True
assert _raised, "sum([1,2], 'x') did not raise TypeError"

# Rule 13: TypeError if iterable contains non-numeric (int + str)
_raised = False
try:
    sum([1, "a"])
except TypeError:
    _raised = True
assert _raised, "sum([1, 'a']) did not raise TypeError"

print("behavior OK")
