# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""int methods: documented exception paths (CPython 3.12 oracle)."""


# int() from non-numeric string raises ValueError.
try:
    int("not_a_number")
    print("bad_str: no_raise")
except ValueError as e:
    print("bad_str:", type(e).__name__, str(e)[:60])


# int() with bad base raises ValueError.
try:
    int("ff", 99)
    print("bad_base: no_raise")
except ValueError as e:
    print("bad_base:", type(e).__name__, str(e)[:60])


# int() with base but no string raises TypeError.
try:
    int(123, 10)  # type: ignore[call-arg]
    print("base_no_str: no_raise")
except TypeError as e:
    print("base_no_str:", type(e).__name__, str(e)[:60])


# Division by zero raises ZeroDivisionError.
try:
    1 // 0
    print("div_zero: no_raise")
except ZeroDivisionError as e:
    print("div_zero:", type(e).__name__, str(e)[:60])


# Modulo by zero raises ZeroDivisionError.
try:
    1 % 0
    print("mod_zero: no_raise")
except ZeroDivisionError as e:
    print("mod_zero:", type(e).__name__, str(e)[:60])


# int.bit_length() doesn't raise.
print("bit_length_0:", (0).bit_length())


# int.from_bytes with bad bytes raises TypeError.
try:
    int.from_bytes("not bytes")  # type: ignore[arg-type]
    print("bad_from_bytes: no_raise")
except TypeError as e:
    print("bad_from_bytes:", type(e).__name__, str(e)[:60])


# int conversion of float infinity raises OverflowError.
try:
    int(float("inf"))
    print("inf_to_int: no_raise")
except OverflowError as e:
    print("inf_to_int:", type(e).__name__, str(e)[:60])


# int conversion of NaN raises ValueError.
try:
    int(float("nan"))
    print("nan_to_int: no_raise")
except ValueError as e:
    print("nan_to_int:", type(e).__name__, str(e)[:60])
