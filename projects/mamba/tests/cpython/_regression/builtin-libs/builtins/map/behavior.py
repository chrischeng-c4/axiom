"""Behavior contract for builtins.map.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: map with lambda over list
result = list(map(lambda x: x * 2, [1, 2, 3]))
assert result == [2, 4, 6], f"map(lambda x*2, [1,2,3]) = {result!r}"

# Rule 2: map with builtin function
result = list(map(str, [1, 2, 3]))
assert result == ["1", "2", "3"], f"map(str, [1,2,3]) = {result!r}"

# Rule 3: map of empty returns empty
result = list(map(str, []))
assert result == [], f"map(str, []) = {result!r}, expected []"

# Rule 4: map returns iterator (not list)
_it = map(str, [1, 2])
assert hasattr(_it, "__next__"), \
    f"map() result has no __next__: type={type(_it).__name__}"

# Rule 5: map with two iterables
result = list(map(lambda x, y: x + y, [1, 2, 3], [10, 20, 30]))
assert result == [11, 22, 33], f"map two iterables = {result!r}"

# Rule 6: map with two iterables truncates to shortest
result = list(map(lambda x, y: x + y, [1, 2, 3], [10, 20]))
assert result == [11, 22], f"map truncates to shortest = {result!r}"

# Rule 7: map is lazy — function not called until iterated
_calls = []
def _track(x):
    _calls.append(x)
    return x
_m = map(_track, [1, 2, 3])
assert _calls == [], f"map eager-evaluated before iteration: {_calls!r}"
next(_m)
assert len(_calls) == 1, f"map called wrong number of times: {_calls!r}"

# Rule 8: map over range
result = list(map(lambda x: x ** 2, range(5)))
assert result == [0, 1, 4, 9, 16], f"map over range = {result!r}"

# Rule 9: map with abs (builtin that works on int)
result = list(map(abs, [-1, -2, 3, -4]))
assert result == [1, 2, 3, 4], f"map(abs, ...) = {result!r}"

# Rule 10: map with None as function is not valid in Python 3 (no filter-like behavior)
_raised = False
try:
    list(map(None, [1, 2, 3]))
except TypeError:
    _raised = True
assert _raised, "map(None, [1,2,3]) did not raise TypeError"

print("behavior OK")
