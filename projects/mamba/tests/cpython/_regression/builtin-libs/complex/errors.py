# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""complex methods: documented exception paths (CPython 3.12 oracle)."""


# complex() from bad string raises ValueError.
try:
    complex("not_a_number")
    print("bad_str: no_raise")
except ValueError as e:
    print("bad_str:", type(e).__name__, str(e)[:60])


# complex() with too many args raises TypeError.
try:
    complex(1, 2, 3)  # type: ignore[call-overload]
    print("three_args: no_raise")
except TypeError as e:
    print("three_args:", type(e).__name__, str(e)[:60])


# complex // operator is unsupported (raises TypeError).
try:
    1j // 2j  # type: ignore[operator]
    print("floor_div: no_raise")
except TypeError as e:
    print("floor_div:", type(e).__name__, str(e)[:60])


# complex % is unsupported.
try:
    1j % 2j  # type: ignore[operator]
    print("mod: no_raise")
except TypeError as e:
    print("mod:", type(e).__name__, str(e)[:60])


# Ordering complex raises TypeError.
try:
    1j < 2j  # type: ignore[operator]
    print("lt: no_raise")
except TypeError as e:
    print("lt:", type(e).__name__, str(e)[:60])


# complex(1) / 0 raises ZeroDivisionError.
try:
    complex(1) / 0
    print("div_zero: no_raise")
except ZeroDivisionError as e:
    print("div_zero:", type(e).__name__, str(e)[:60])


# Mixed type ops with str raise TypeError.
try:
    1j + "x"  # type: ignore[operator]
    print("plus_str: no_raise")
except TypeError as e:
    print("plus_str:", type(e).__name__, str(e)[:60])


# Adding an int too large to fit a C double raises OverflowError.
try:
    1j + 10 ** 1000
    print("huge_int: no_raise")
except OverflowError as e:
    print("huge_int:", type(e).__name__, str(e)[:60])


# Arithmetic with None raises TypeError (both operand orders).
try:
    1j * None  # type: ignore[operator]
    print("mul_none: no_raise")
except TypeError as e:
    print("mul_none:", type(e).__name__, str(e)[:60])


# divmod() on complex is unsupported (TypeError, not ZeroDivisionError).
try:
    divmod(1 + 1j, 1)  # type: ignore[arg-type]
    print("divmod: no_raise")
except TypeError as e:
    print("divmod:", type(e).__name__, str(e)[:60])


# Happy: complex arith.
print("conj:", (1 + 2j).conjugate())
print("abs:", abs(3 + 4j))
print("real:", (1 + 2j).real, "imag:", (1 + 2j).imag)
