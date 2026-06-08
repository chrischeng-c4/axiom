# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""float methods: documented exception paths (CPython 3.12 oracle)."""


# float() from non-numeric raises ValueError.
try:
    float("hello")
    print("bad_str: no_raise")
except ValueError as e:
    print("bad_str:", type(e).__name__, str(e)[:60])


# Division by zero raises ZeroDivisionError.
try:
    1.0 / 0
    print("div_zero: no_raise")
except ZeroDivisionError as e:
    print("div_zero:", type(e).__name__, str(e)[:60])


# 0.0 ** -1 raises ZeroDivisionError.
try:
    0.0 ** -1
    print("zero_neg_pow: no_raise")
except ZeroDivisionError as e:
    print("zero_neg_pow:", type(e).__name__, str(e)[:60])


# float.fromhex with bad hex raises ValueError.
try:
    float.fromhex("not_hex")
    print("bad_hex: no_raise")
except ValueError as e:
    print("bad_hex:", type(e).__name__, str(e)[:60])


# float.as_integer_ratio on inf raises OverflowError.
try:
    float("inf").as_integer_ratio()
    print("inf_ratio: no_raise")
except OverflowError as e:
    print("inf_ratio:", type(e).__name__, str(e)[:60])


# float.as_integer_ratio on NaN raises ValueError.
try:
    float("nan").as_integer_ratio()
    print("nan_ratio: no_raise")
except ValueError as e:
    print("nan_ratio:", type(e).__name__, str(e)[:60])


# Mixed type op raises TypeError.
try:
    1.0 + "x"  # type: ignore[operator]
    print("float_plus_str: no_raise")
except TypeError as e:
    print("float_plus_str:", type(e).__name__, str(e)[:60])


# round with bad ndigits raises TypeError.
try:
    round(1.5, "two")  # type: ignore[arg-type]
    print("bad_ndigits: no_raise")
except TypeError as e:
    print("bad_ndigits:", type(e).__name__, str(e)[:60])
