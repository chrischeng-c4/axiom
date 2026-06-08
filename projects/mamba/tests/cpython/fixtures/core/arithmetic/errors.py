# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/<area>: documented language-level error paths (CPython 3.12 oracle)."""


# Division by zero raises ZeroDivisionError.
try:
    _ = 1 // 0
    print("div_zero: no_raise")
except ZeroDivisionError as e:
    print("div_zero:", type(e).__name__, str(e)[:60])


# Mixed-type op raises TypeError.
try:
    _ = 1 + "a"  # type: ignore[operator]
    print("mixed: no_raise")
except TypeError as e:
    print("mixed:", type(e).__name__, str(e)[:60])


# Comparing mixed unorderable types raises TypeError.
try:
    _ = 1 < "a"  # type: ignore[operator]
    print("cmp_mixed: no_raise")
except TypeError as e:
    print("cmp_mixed:", type(e).__name__, str(e)[:60])
