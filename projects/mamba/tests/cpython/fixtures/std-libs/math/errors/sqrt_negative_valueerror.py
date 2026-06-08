# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "sqrt_negative_valueerror"
# subject = "math.sqrt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.sqrt: sqrt_negative_valueerror (errors)."""
import math

_raised = False
try:
    math.sqrt(-1)
except ValueError:
    _raised = True
assert _raised, "sqrt_negative_valueerror: expected ValueError"
print("sqrt_negative_valueerror OK")
