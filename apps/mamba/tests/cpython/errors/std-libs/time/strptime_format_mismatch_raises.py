# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "errors"
# case = "strptime_format_mismatch_raises"
# subject = "time.strptime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.strptime: strptime_format_mismatch_raises (errors)."""
import time

_raised = False
try:
    time.strptime('not_a_date', '%Y-%m-%d')
except ValueError:
    _raised = True
assert _raised, "strptime_format_mismatch_raises: expected ValueError"
print("strptime_format_mismatch_raises OK")
