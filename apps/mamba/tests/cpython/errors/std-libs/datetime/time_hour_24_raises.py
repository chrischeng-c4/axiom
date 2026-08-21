# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "time_hour_24_raises"
# subject = "datetime.time"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.time: time_hour_24_raises (errors)."""
import datetime

_raised = False
try:
    datetime.time(24, 0, 0)
except ValueError:
    _raised = True
assert _raised, "time_hour_24_raises: expected ValueError"
print("time_hour_24_raises OK")
