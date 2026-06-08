# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "log_zero_valueerror"
# subject = "math.log"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.log: log_zero_valueerror (errors)."""
import math

_raised = False
try:
    math.log(0)
except ValueError:
    _raised = True
assert _raised, "log_zero_valueerror: expected ValueError"
print("log_zero_valueerror OK")
