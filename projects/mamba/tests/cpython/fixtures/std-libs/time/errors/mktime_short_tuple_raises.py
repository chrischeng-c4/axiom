# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "errors"
# case = "mktime_short_tuple_raises"
# subject = "time.mktime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_time.py"
# status = "filled"
# ///
"""time.mktime: mktime_short_tuple_raises (errors)."""
import time

_raised = False
try:
    time.mktime((2024, 1, 1))
except TypeError:
    _raised = True
assert _raised, "mktime_short_tuple_raises: expected TypeError"
print("mktime_short_tuple_raises OK")
