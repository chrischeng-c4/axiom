# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "sumprod_length_mismatch_valueerror"
# subject = "math.sumprod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.sumprod: sumprod_length_mismatch_valueerror (errors)."""
import math

_raised = False
try:
    math.sumprod([1, 2], [3])
except ValueError:
    _raised = True
assert _raised, "sumprod_length_mismatch_valueerror: expected ValueError"
print("sumprod_length_mismatch_valueerror OK")
