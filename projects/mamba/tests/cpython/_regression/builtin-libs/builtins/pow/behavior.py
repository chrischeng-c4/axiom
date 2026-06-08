"""Behavior contract for builtins.pow.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import math

# Rule 1: integer exponentiation
assert pow(2, 0) == 1, f"pow(2,0) = {pow(2,0)!r}"
assert pow(2, 1) == 2, f"pow(2,1) = {pow(2,1)!r}"
assert pow(2, 10) == 1024, f"pow(2,10) = {pow(2,10)!r}"
assert pow(3, 3) == 27, f"pow(3,3) = {pow(3,3)!r}"

# Rule 2: ** operator matches pow()
assert 2 ** 10 == pow(2, 10), f"** != pow"

# Rule 3: negative exponent with int returns float
assert pow(2, -1) == 0.5, f"pow(2,-1) = {pow(2,-1)!r}"
assert isinstance(pow(2, -1), float), f"pow(int,neg) should be float"

# Rule 4: float exponentiation
assert pow(4.0, 0.5) == 2.0, f"pow(4.0,0.5) = {pow(4.0,0.5)!r}"
assert abs(pow(2.0, 0.5) - math.sqrt(2)) < 1e-10, "pow(2.0,0.5) != sqrt(2)"

# Rule 5: three-arg pow(base, exp, mod) — modular exponentiation
assert pow(2, 10, 1000) == 24, f"pow(2,10,1000) = {pow(2,10,1000)!r}"
assert pow(3, 5, 7) == 5, f"pow(3,5,7) = {pow(3,5,7)!r}"
assert pow(0, 0, 7) == 1, f"pow(0,0,7) = {pow(0,0,7)!r}"

# Rule 6: three-arg requires integers
_raised = False
try:
    pow(2.0, 3, 5)  # type: ignore[misc]
except TypeError:
    _raised = True
assert _raised, "pow(float,int,int) did not raise TypeError"

# Rule 7: three-arg with zero mod raises ValueError
_raised = False
try:
    pow(2, 10, 0)
except ValueError:
    _raised = True
assert _raised, "pow(2,10,0) did not raise ValueError"

# Rule 8: pow(0, 0) == 1 (by convention)
assert pow(0, 0) == 1, f"pow(0,0) = {pow(0,0)!r}"
assert pow(0.0, 0.0) == 1.0, f"pow(0.0,0.0) = {pow(0.0,0.0)!r}"

# Rule 9: custom __pow__
class _Num:
    def __init__(self, v: int):
        self.v = v
    def __pow__(self, other: "_Num") -> int:
        return self.v ** other.v
a = _Num(3)
b = _Num(4)
assert a ** b == 81, f"custom __pow__ = {a ** b!r}"

print("behavior OK")
