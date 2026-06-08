"""Behavior contract for builtins.divmod.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: basic int divmod
q, r = divmod(17, 5)
assert q == 3, f"q = {q!r}"
assert r == 2, f"r = {r!r}"
assert 5 * q + r == 17, "invariant: b*q + r == a"

# Rule 2: negative dividend
q, r = divmod(-17, 5)
assert q == -4, f"q(-17,5) = {q!r}"  # floor division rounds toward -inf
assert r == 3, f"r(-17,5) = {r!r}"
assert 5 * q + r == -17, "invariant -17,5"

# Rule 3: negative divisor
q, r = divmod(17, -5)
assert q == -4, f"q(17,-5) = {q!r}"
assert r == -3, f"r(17,-5) = {r!r}"
assert -5 * q + r == 17, "invariant 17,-5"

# Rule 4: both negative
q, r = divmod(-17, -5)
assert q == 3, f"q(-17,-5) = {q!r}"
assert r == -2, f"r(-17,-5) = {r!r}"
assert -5 * q + r == -17, "invariant -17,-5"

# Rule 5: float divmod
q, r = divmod(7.5, 2.5)
assert q == 3.0, f"q(7.5,2.5) = {q!r}"
assert abs(r) < 1e-10, f"r(7.5,2.5) ≈ 0: {r!r}"

# Rule 6: divmod with zero divisor raises ZeroDivisionError
_raised = False
try:
    divmod(5, 0)
except ZeroDivisionError:
    _raised = True
assert _raised, "divmod(5,0) did not raise ZeroDivisionError"

# Rule 7: float zero divisor
_raised = False
try:
    divmod(5.0, 0.0)
except ZeroDivisionError:
    _raised = True
assert _raised, "divmod(5.0,0.0) did not raise ZeroDivisionError"

# Rule 8: divmod result satisfies b*q + r == a (invariant)
for a, b in [(100, 7), (-100, 7), (100, -7), (0, 1), (1, 1)]:
    q2, r2 = divmod(a, b)
    assert b * q2 + r2 == a, f"invariant failed for divmod({a},{b})"

# Rule 9: remainder sign matches divisor sign (floor semantics)
q3, r3 = divmod(-1, 3)
assert r3 >= 0, f"r(-1,3) should be >= 0: {r3!r}"
q4, r4 = divmod(1, -3)
assert r4 <= 0, f"r(1,-3) should be <= 0: {r4!r}"

print("behavior OK")
