# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "atan2_bad_second_typeerror"
# subject = "math.atan2"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.atan2: atan2_bad_second_typeerror (errors)."""
import math

_raised = False
try:
    math.atan2(1.0, "spam")
except TypeError:
    _raised = True
assert _raised, "atan2_bad_second_typeerror: expected TypeError"
print("atan2_bad_second_typeerror OK")
