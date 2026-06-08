# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "factorial_float_typeerror"
# subject = "math.factorial"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.factorial: factorial_float_typeerror (errors)."""
import math

_raised = False
try:
    math.factorial(3.5)
except TypeError:
    _raised = True
assert _raised, "factorial_float_typeerror: expected TypeError"
print("factorial_float_typeerror OK")
