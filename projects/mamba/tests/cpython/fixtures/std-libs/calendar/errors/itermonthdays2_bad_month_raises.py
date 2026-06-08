# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "itermonthdays2_bad_month_raises"
# subject = "calendar.Calendar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.Calendar: itermonthdays2_bad_month_raises (errors)."""
import calendar

_raised = False
try:
    list(calendar.Calendar().itermonthdays2(2024, 13))
except calendar.IllegalMonthError:
    _raised = True
assert _raised, "itermonthdays2_bad_month_raises: expected calendar.IllegalMonthError"
print("itermonthdays2_bad_month_raises OK")
