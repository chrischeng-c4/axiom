"""Behavior contract for builtins.any.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: at least one truthy
assert any([False, True, False]) == True, "any([F,T,F]) expected True"

# Rule 2: all falsy
assert any([False, False, False]) == False, "any([F,F,F]) expected False"

# Rule 3: empty iterable returns False (vacuous false)
assert any([]) == False, "any([]) expected False"

# Rule 4: any non-zero int is truthy
assert any([0, 0, 1]) == True, "any([0,0,1]) expected True"

# Rule 5: all zeros → False
assert any([0, 0, 0]) == False, "any([0,0,0]) expected False"

# Rule 6: returns bool
result = any([1, 2, 3])
assert type(result) is bool, f"type(any(...)) = {type(result).__name__!r}, expected 'bool'"

# Rule 7: short-circuit — stops at first truthy
_evals = []
def _track(x):
    _evals.append(x)
    return x
any(_track(x) for x in [0, 1, 2, 3])
assert 2 not in _evals, f"any() did not short-circuit: evaluated {_evals!r}"

# Rule 8: any over generator expression
assert any(x > 0 for x in [-1, -2, 1]) == True
assert any(x > 0 for x in [-1, -2, -3]) == False

# Rule 9: any with strings
assert any(["", "", "a"]) == True
assert any(["", "", ""]) == False

# Rule 10: TypeError for non-iterable
_raised = False
try:
    any(42)
except TypeError:
    _raised = True
assert _raised, "any(42) did not raise TypeError"

print("behavior OK")
