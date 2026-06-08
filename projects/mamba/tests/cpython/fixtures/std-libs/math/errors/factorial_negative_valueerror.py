# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "factorial_negative_valueerror"
# subject = "math.factorial"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.factorial: factorial_negative_valueerror (errors)."""
import math

_raised = False
try:
    math.factorial(-1)
except ValueError:
    _raised = True
assert _raised, "factorial_negative_valueerror: expected ValueError"
print("factorial_negative_valueerror OK")
