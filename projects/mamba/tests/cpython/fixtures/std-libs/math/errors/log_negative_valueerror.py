# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "errors"
# case = "log_negative_valueerror"
# subject = "math.log"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.log: log_negative_valueerror (errors)."""
import math

_raised = False
try:
    math.log(-1)
except ValueError:
    _raised = True
assert _raised, "log_negative_valueerror: expected ValueError"
print("log_negative_valueerror OK")
