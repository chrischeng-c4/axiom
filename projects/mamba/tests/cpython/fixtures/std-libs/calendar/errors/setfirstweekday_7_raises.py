# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "setfirstweekday_7_raises"
# subject = "calendar.setfirstweekday"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.setfirstweekday: setfirstweekday_7_raises (errors)."""
import calendar

_raised = False
try:
    calendar.setfirstweekday(7)
except ValueError:
    _raised = True
assert _raised, "setfirstweekday_7_raises: expected ValueError"
print("setfirstweekday_7_raises OK")
