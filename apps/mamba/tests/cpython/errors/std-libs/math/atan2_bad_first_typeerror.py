# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "atan2_bad_first_typeerror"
# subject = "math.atan2"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.atan2: atan2_bad_first_typeerror (errors)."""
import math

_raised = False
try:
    math.atan2("spam", 1.0)
except TypeError:
    _raised = True
assert _raised, "atan2_bad_first_typeerror: expected TypeError"
print("atan2_bad_first_typeerror OK")
