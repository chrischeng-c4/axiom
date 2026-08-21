# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "errors"
# case = "asctime_int_arg_raises"
# subject = "time.asctime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.asctime: asctime_int_arg_raises (errors)."""
import time

_raised = False
try:
    time.asctime(123)
except TypeError:
    _raised = True
assert _raised, "asctime_int_arg_raises: expected TypeError"
print("asctime_int_arg_raises OK")
