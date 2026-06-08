# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "errors"
# case = "abs_overflow_raises_overflowerror"
# subject = "abs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
"""abs: abs_overflow_raises_overflowerror (errors)."""
import cmath  # noqa: F401

_raised = False
try:
    abs(complex(1.4e308, 1.4e308))
except OverflowError:
    _raised = True
assert _raised, "abs_overflow_raises_overflowerror: expected OverflowError"
print("abs_overflow_raises_overflowerror OK")
