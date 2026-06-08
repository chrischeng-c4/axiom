# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "dist_length_mismatch_valueerror"
# subject = "math.dist"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.dist: dist_length_mismatch_valueerror (errors)."""
import math

_raised = False
try:
    math.dist([1, 2], [3, 4, 5])
except ValueError:
    _raised = True
assert _raised, "dist_length_mismatch_valueerror: expected ValueError"
print("dist_length_mismatch_valueerror OK")
