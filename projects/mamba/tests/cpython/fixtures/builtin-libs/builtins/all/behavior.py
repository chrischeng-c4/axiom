"""Behavior contract for builtins.all.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: all truthy
assert all([True, True, True]) == True, "all([T,T,T]) expected True"

# Rule 2: one falsy
assert all([True, False, True]) == False, "all([T,F,T]) expected False"

# Rule 3: empty iterable returns True (vacuous truth)
assert all([]) == True, "all([]) expected True"

# Rule 4: all non-zero ints are truthy
assert all([1, 2, 3]) == True, "all([1,2,3]) expected True"

# Rule 5: zero is falsy
assert all([1, 0, 2]) == False, "all([1,0,2]) expected False"

# Rule 6: returns bool
result = all([1, 2, 3])
assert type(result) is bool, f"type(all(...)) = {type(result).__name__!r}, expected 'bool'"

# Rule 7: short-circuit — stops at first falsy
_evals = []
def _track(x):
    _evals.append(x)
    return x
all(_track(x) for x in [1, 0, 2, 3])
assert 2 not in _evals, f"all() did not short-circuit: evaluated {_evals!r}"

# Rule 8: all over generator expression
assert all(x > 0 for x in [1, 2, 3]) == True
assert all(x > 0 for x in [1, -1, 2]) == False

# Rule 9: all with strings (non-empty str is truthy)
assert all(["a", "b", "c"]) == True
assert all(["a", "", "c"]) == False

# Rule 10: TypeError for non-iterable
_raised = False
try:
    all(42)
except TypeError:
    _raised = True
assert _raised, "all(42) did not raise TypeError"

print("behavior OK")
