# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "leapdays_is_callable"
# subject = "calendar.leapdays"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""calendar.leapdays: leapdays_is_callable (surface)."""
import calendar

assert callable(calendar.leapdays)
print("leapdays_is_callable OK")
