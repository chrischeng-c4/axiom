# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "errors"
# case = "log_zero_raises_valueerror"
# subject = "cmath.log"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
"""cmath.log: log_zero_raises_valueerror (errors)."""
import cmath

_raised = False
try:
    cmath.log(0)
except ValueError:
    _raised = True
assert _raised, "log_zero_raises_valueerror: expected ValueError"
print("log_zero_raises_valueerror OK")
