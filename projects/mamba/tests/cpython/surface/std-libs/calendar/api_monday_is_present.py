# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_monday_is_present"
# subject = "calendar.MONDAY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.MONDAY: api_monday_is_present (surface)."""
import calendar

assert hasattr(calendar, "MONDAY")
print("api_monday_is_present OK")
