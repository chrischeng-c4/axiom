# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_setfirstweekday_is_present"
# subject = "calendar.setfirstweekday"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.setfirstweekday: api_setfirstweekday_is_present (surface)."""
import calendar

assert hasattr(calendar, "setfirstweekday")
print("api_setfirstweekday_is_present OK")
