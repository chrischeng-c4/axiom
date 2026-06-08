# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_weekday_is_present"
# subject = "calendar.weekday"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.weekday: api_weekday_is_present (surface)."""
import calendar

assert hasattr(calendar, "weekday")
print("api_weekday_is_present OK")
