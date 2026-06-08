"""Behavior contract for builtins.range.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: range(n) — 0 to n-1
result = list(range(5))
assert result == [0, 1, 2, 3, 4], f"list(range(5)) = {result!r}"

# Rule 2: range(start, stop)
result = list(range(2, 8))
assert result == [2, 3, 4, 5, 6, 7], f"list(range(2,8)) = {result!r}"

# Rule 3: range(start, stop, step)
result = list(range(0, 10, 2))
assert result == [0, 2, 4, 6, 8], f"list(range(0,10,2)) = {result!r}"

# Rule 4: reverse range
result = list(range(5, 0, -1))
assert result == [5, 4, 3, 2, 1], f"list(range(5,0,-1)) = {result!r}"

# Rule 5: empty range (start >= stop, positive step)
result = list(range(5, 2))
assert result == [], f"list(range(5,2)) = {result!r}, expected []"

# Rule 6: len of range
assert len(range(5)) == 5, f"len(range(5)) = {len(range(5))!r}"
assert len(range(0, 10, 2)) == 5, f"len(range(0,10,2)) = {len(range(0,10,2))!r}"
assert len(range(5, 2)) == 0, f"len(range(5,2)) = {len(range(5,2))!r}"

# Rule 7: membership test
assert (3 in range(5)) == True, f"3 in range(5) = {3 in range(5)!r}"
assert (5 in range(5)) == False, f"5 in range(5) = {5 in range(5)!r}"
assert (3 in range(0, 10, 2)) == False, f"3 in range(0,10,2) = {3 in range(0,10,2)!r}"

# Rule 8: indexing
assert range(5)[0] == 0, f"range(5)[0] = {range(5)[0]!r}"
assert range(5)[4] == 4, f"range(5)[4] = {range(5)[4]!r}"
assert range(5)[-1] == 4, f"range(5)[-1] = {range(5)[-1]!r}"

# Rule 9: slicing returns range object
_s = range(10)[2:7:2]
assert list(_s) == [2, 4, 6], f"range(10)[2:7:2] = {list(_s)!r}"
assert type(_s) is range, f"type(range slice) = {type(_s).__name__!r}, expected 'range'"

# Rule 10: range is iterable (supports iter protocol)
_r = range(3)
_it = iter(_r)
assert next(_it) == 0
assert next(_it) == 1
assert next(_it) == 2

# Rule 11: TypeError for zero step
_raised = False
try:
    range(0, 10, 0)
except ValueError:
    _raised = True
assert _raised, "range(0, 10, 0) did not raise ValueError"

# Rule 12: range equality
assert range(5) == range(0, 5), f"range(5) == range(0,5)"
assert range(5) != range(6), f"range(5) != range(6)"

print("behavior OK")
