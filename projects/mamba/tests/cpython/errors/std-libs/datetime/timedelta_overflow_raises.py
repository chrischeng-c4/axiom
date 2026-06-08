# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "timedelta_overflow_raises"
# subject = "datetime.timedelta"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.timedelta: timedelta_overflow_raises (errors)."""
import datetime

_raised = False
try:
    datetime.timedelta(days=10**10) + datetime.timedelta(days=10**10)
except OverflowError:
    _raised = True
assert _raised, "timedelta_overflow_raises: expected OverflowError"
print("timedelta_overflow_raises OK")
