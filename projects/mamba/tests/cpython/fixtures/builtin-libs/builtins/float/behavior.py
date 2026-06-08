"""Behavior contract for builtins.float.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import math

# Rule 1: float() with no args returns 0.0
assert float() == 0.0, f"float() = {float()!r}"

# Rule 2: float(int)
assert float(3) == 3.0, f"float(3) = {float(3)!r}"
assert float(-1) == -1.0, f"float(-1) = {float(-1)!r}"

# Rule 3: float(float) is identity-like
assert float(3.14) == 3.14, f"float(3.14) = {float(3.14)!r}"

# Rule 4: float(str) parses decimal
assert float("3.14") == 3.14, f"float('3.14') = {float('3.14')!r}"
assert float("-2.5") == -2.5, f"float('-2.5') = {float('-2.5')!r}"
assert float("  1.0  ") == 1.0, f"float('  1.0  ') = {float('  1.0  ')!r}"

# Rule 5: float("inf") and float("nan")
assert math.isinf(float("inf")), "float('inf') not inf"
assert math.isinf(float("-inf")), "float('-inf') not -inf"
assert float("inf") > 0, "float('inf') not positive"
assert math.isnan(float("nan")), "float('nan') not nan"

# Rule 6: float("abc") raises ValueError
_raised = False
try:
    float("abc")
except ValueError:
    _raised = True
assert _raised, "float('abc') did not raise ValueError"

# Rule 7: float(None) raises TypeError
_raised = False
try:
    float(None)
except TypeError:
    _raised = True
assert _raised, "float(None) did not raise TypeError"

# Rule 8: arithmetic returns float
assert type(1.0 + 2.0) is float, f"type(1.0+2.0) = {type(1.0+2.0).__name__!r}"
assert type(3.0 * 2.0) is float, f"type(3.0*2.0) = {type(3.0*2.0).__name__!r}"
assert type(7.0 / 2.0) is float, f"type(7.0/2.0) = {type(7.0/2.0).__name__!r}"
assert type(int(1) + float(2.0)) is float, "int+float should yield float"

# Rule 9: int + float yields float
assert 1 + 1.0 == 2.0, f"1+1.0 = {1+1.0!r}"
assert type(1 + 1.0) is float, f"type(1+1.0) = {type(1+1.0).__name__!r}"

# Rule 10: float.is_integer()
assert (3.0).is_integer() is True, f"(3.0).is_integer() = {(3.0).is_integer()!r}"
assert (3.5).is_integer() is False, f"(3.5).is_integer() = {(3.5).is_integer()!r}"

# Rule 11: float.as_integer_ratio()
n, d = (0.25).as_integer_ratio()
assert n == 1 and d == 4, f"(0.25).as_integer_ratio() = {(0.25).as_integer_ratio()!r}"

# Rule 12: float.hex() and float.fromhex()
assert float.fromhex("0x1.8p+1") == 3.0, \
    f"float.fromhex('0x1.8p+1') = {float.fromhex('0x1.8p+1')!r}"

# Rule 13: nan comparison
nan = float("nan")
assert not (nan == nan), "nan == nan should be False"
assert nan != nan, "nan != nan should be True"

# Rule 14: inf arithmetic
inf = float("inf")
assert inf + 1 == inf, f"inf+1 = {inf+1!r}"
assert inf * 2 == inf, f"inf*2 = {inf*2!r}"
assert math.isnan(inf - inf), f"inf-inf should be nan"

print("behavior OK")
