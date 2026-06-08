# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "setfirstweekday_negative_valueerror"
# subject = "calendar.setfirstweekday"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.setfirstweekday: setfirstweekday_negative_valueerror (errors)."""
import calendar

_raised = False
try:
    calendar.setfirstweekday(-1)
except ValueError:
    _raised = True
assert _raised, "setfirstweekday_negative_valueerror: expected ValueError"
print("setfirstweekday_negative_valueerror OK")
