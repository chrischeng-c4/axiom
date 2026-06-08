# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "day_name_index_7_raises"
# subject = "calendar.day_name"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.day_name: day_name_index_7_raises (errors)."""
import calendar

_raised = False
try:
    calendar.day_name[7]
except IndexError:
    _raised = True
assert _raised, "day_name_index_7_raises: expected IndexError"
print("day_name_index_7_raises OK")
