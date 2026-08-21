# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "sqrt_str_typeerror"
# subject = "math.sqrt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.sqrt: sqrt_str_typeerror (errors)."""
import math

_raised = False
try:
    math.sqrt("hello")
except TypeError:
    _raised = True
assert _raised, "sqrt_str_typeerror: expected TypeError"
print("sqrt_str_typeerror OK")
