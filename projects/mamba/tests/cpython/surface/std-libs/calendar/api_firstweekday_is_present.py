# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_firstweekday_is_present"
# subject = "calendar.firstweekday"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.firstweekday: api_firstweekday_is_present (surface)."""
import calendar

assert hasattr(calendar, "firstweekday")
print("api_firstweekday_is_present OK")
