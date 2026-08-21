# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "errors"
# case = "setfirstweekday_str_typeerror"
# subject = "calendar.setfirstweekday"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_calendar.py"
# status = "filled"
# ///
"""calendar.setfirstweekday: setfirstweekday_str_typeerror (errors)."""
import calendar

_raised = False
try:
    calendar.setfirstweekday("flabber")
except TypeError:
    _raised = True
assert _raised, "setfirstweekday_str_typeerror: expected TypeError"
print("setfirstweekday_str_typeerror OK")
