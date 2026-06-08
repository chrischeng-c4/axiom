# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "weekday_is_callable"
# subject = "calendar.weekday"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.weekday: weekday_is_callable (surface)."""
import calendar

assert callable(calendar.weekday)
print("weekday_is_callable OK")
