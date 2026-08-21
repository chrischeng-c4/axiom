# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "errors"
# case = "clock_gettime_bad_id_raises"
# subject = "time.clock_gettime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.clock_gettime: clock_gettime_bad_id_raises (errors)."""
import time

_raised = False
try:
    time.clock_gettime(999999)
except OSError:
    _raised = True
assert _raised, "clock_gettime_bad_id_raises: expected OSError"
print("clock_gettime_bad_id_raises OK")
