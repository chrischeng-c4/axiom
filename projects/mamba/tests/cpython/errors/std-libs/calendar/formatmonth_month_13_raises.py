# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "formatmonth_month_13_raises"
# subject = "calendar.TextCalendar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.TextCalendar: formatmonth_month_13_raises (errors)."""
import calendar

_raised = False
try:
    calendar.TextCalendar().formatmonth(2017, 13)
except calendar.IllegalMonthError:
    _raised = True
assert _raised, "formatmonth_month_13_raises: expected calendar.IllegalMonthError"
print("formatmonth_month_13_raises OK")
