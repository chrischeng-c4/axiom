# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "day_abbr_negative_index_raises"
# subject = "calendar.day_abbr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.day_abbr: day_abbr_negative_index_raises (errors)."""
import calendar

_raised = False
try:
    calendar.day_abbr[-10]
except IndexError:
    _raised = True
assert _raised, "day_abbr_negative_index_raises: expected IndexError"
print("day_abbr_negative_index_raises OK")
