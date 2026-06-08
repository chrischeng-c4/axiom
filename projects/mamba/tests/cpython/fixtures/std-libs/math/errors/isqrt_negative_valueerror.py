# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "isqrt_negative_valueerror"
# subject = "math.isqrt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.isqrt: isqrt_negative_valueerror (errors)."""
import math

_raised = False
try:
    math.isqrt(-1)
except ValueError:
    _raised = True
assert _raised, "isqrt_negative_valueerror: expected ValueError"
print("isqrt_negative_valueerror OK")
