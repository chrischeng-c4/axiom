# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "erf_str_typeerror"
# subject = "math.erf"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.erf: erf_str_typeerror (errors)."""
import math

_raised = False
try:
    math.erf("spam")
except TypeError:
    _raised = True
assert _raised, "erf_str_typeerror: expected TypeError"
print("erf_str_typeerror OK")
