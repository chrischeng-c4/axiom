"""Behavior contract for language comparison operators.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: equality / inequality
assert (1 == 1) == True
assert (1 == 2) == False
assert (1 != 2) == True
assert (1 != 1) == False
assert ("abc" == "abc") == True
assert ("abc" == "xyz") == False

# Rule 2: ordered comparisons on ints
assert (1 < 2) == True
assert (2 < 1) == False
assert (1 <= 1) == True
assert (1 <= 0) == False
assert (3 > 2) == True
assert (2 > 3) == False
assert (3 >= 3) == True
assert (2 >= 3) == False

# Rule 3: string comparison (lexicographic)
assert ("a" < "b") == True
assert ("b" < "a") == False
assert ("abc" < "abd") == True
assert ("abc" == "abc") == True
assert ("z" > "a") == True

# Rule 4: chained comparisons (short-circuit AND)
assert (1 < 2 < 3) == True
assert (1 < 2 > 3) == False
assert (0 < 1 < 2 < 3 < 4) == True
assert (0 <= 0 <= 1) == True
assert (1 < 2 < 2) == False

# Rule 5: chained comparisons evaluate middle exactly once
_calls = []
def _val(x: int) -> int:
    _calls.append(x)
    return x
assert (1 < _val(2) < 3) == True
assert len(_calls) == 1, f"middle evaluated {len(_calls)} times, expected 1"

# Rule 6: is / is not — identity, not equality
a = [1, 2]
b = [1, 2]
c = a
assert (a is c) == True
assert (a is b) == False
assert (a is not b) == True
assert (a is not c) == False

# Rule 7: None comparisons
x = None
assert (x is None) == True
assert (x is not None) == False

# Rule 8: in / not in
lst = [1, 2, 3]
assert (1 in lst) == True
assert (4 in lst) == False
assert (4 not in lst) == True
assert ("a" in "cat") == True
assert ("z" in "cat") == False

# Rule 9: comparison of different numeric types
assert (1 == 1.0) == True     # int and float equality
assert (1 < 1.5) == True
assert (2.0 > 1) == True
assert (1 == True) == True    # bool/int

# Rule 10: None is not comparable with < / >
_raised = False
try:
    result = None < 1  # type: ignore[operator]
except TypeError:
    _raised = True
assert _raised, "None < 1 did not raise TypeError"

# Rule 11: custom __eq__
class _Eq:
    def __eq__(self, other: object) -> bool:
        return True  # always equal
assert (_Eq() == 42) == True, "custom __eq__ failed"

print("behavior OK")
