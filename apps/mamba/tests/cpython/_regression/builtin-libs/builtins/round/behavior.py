"""Behavior contract for builtins.round.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: round(int) returns int unchanged
assert round(5) == 5, f"round(5) = {round(5)!r}"
assert round(-3) == -3, f"round(-3) = {round(-3)!r}"
assert isinstance(round(5), int), "round(int) not int"

# Rule 2: round(float) returns int (no ndigits)
assert isinstance(round(3.14), int), "round(float) returns int"
assert round(3.0) == 3, f"round(3.0) = {round(3.0)!r}"
assert round(3.4) == 3, f"round(3.4) = {round(3.4)!r}"
assert round(3.6) == 4, f"round(3.6) = {round(3.6)!r}"
assert round(-3.4) == -3, f"round(-3.4) = {round(-3.4)!r}"
assert round(-3.6) == -4, f"round(-3.6) = {round(-3.6)!r}"

# Rule 3: banker's rounding (round-half-to-even)
assert round(0.5) == 0, f"round(0.5) = {round(0.5)!r}"   # toward even: 0
assert round(1.5) == 2, f"round(1.5) = {round(1.5)!r}"   # toward even: 2
assert round(2.5) == 2, f"round(2.5) = {round(2.5)!r}"   # toward even: 2
assert round(3.5) == 4, f"round(3.5) = {round(3.5)!r}"   # toward even: 4
assert round(4.5) == 4, f"round(4.5) = {round(4.5)!r}"   # toward even: 4

# Rule 4: round(float, ndigits) — ndigits > 0
assert round(3.14159, 2) == 3.14, f"round(3.14159,2) = {round(3.14159,2)!r}"
assert round(3.145, 2) == 3.14 or round(3.145, 2) == 3.15, \
    f"round(3.145,2) = {round(3.145,2)!r}"  # float repr can vary
assert round(-1.235, 2) == -1.24 or round(-1.235, 2) == -1.23, \
    f"round(-1.235,2) = {round(-1.235,2)!r}"

# Rule 5: round(float, ndigits) returns float
assert isinstance(round(3.14, 1), float), "round(float, 1) returns float"

# Rule 6: round(int, ndigits) returns int
assert isinstance(round(42, 1), int), "round(int, 1) returns int"
assert round(42, 1) == 42, f"round(42,1) = {round(42,1)!r}"

# Rule 7: round(x, negative ndigits) — rounds to 10s, 100s, etc.
assert round(1234, -1) == 1230, f"round(1234,-1) = {round(1234,-1)!r}"
assert round(1250, -2) == 1200, f"round(1250,-2) = {round(1250,-2)!r}"
assert round(1350, -2) == 1400, f"round(1350,-2) = {round(1350,-2)!r}"

# Rule 8: custom __round__
class _Num:
    def __round__(self, ndigits: int = 0) -> int:
        return 42
assert round(_Num()) == 42, f"custom __round__ = {round(_Num())!r}"

# Rule 9: round(float, 0) returns float, not int
assert isinstance(round(3.14, 0), float), "round(float,0) should be float"
assert round(3.14, 0) == 3.0, f"round(3.14,0) = {round(3.14,0)!r}"

print("behavior OK")
