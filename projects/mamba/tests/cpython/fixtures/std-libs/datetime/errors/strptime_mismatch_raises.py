# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "errors"
# case = "strptime_mismatch_raises"
# subject = "datetime.datetime.strptime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.datetime.strptime: strptime_mismatch_raises (errors)."""
import datetime

_raised = False
try:
    datetime.datetime.strptime("not_a_date", "%Y-%m-%d")
except ValueError:
    _raised = True
assert _raised, "strptime_mismatch_raises: expected ValueError"
print("strptime_mismatch_raises OK")
