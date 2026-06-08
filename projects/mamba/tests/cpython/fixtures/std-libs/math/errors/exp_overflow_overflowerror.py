# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "exp_overflow_overflowerror"
# subject = "math.exp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.exp: exp_overflow_overflowerror (errors)."""
import math

_raised = False
try:
    math.exp(1e9)
except OverflowError:
    _raised = True
assert _raised, "exp_overflow_overflowerror: expected OverflowError"
print("exp_overflow_overflowerror OK")
