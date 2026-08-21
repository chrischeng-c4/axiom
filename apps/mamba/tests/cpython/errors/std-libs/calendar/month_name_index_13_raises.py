# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "month_name_index_13_raises"
# subject = "calendar.month_name"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.month_name: month_name_index_13_raises (errors)."""
import calendar

_raised = False
try:
    calendar.month_name[13]
except IndexError:
    _raised = True
assert _raised, "month_name_index_13_raises: expected IndexError"
print("month_name_index_13_raises OK")
