# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_calendar_is_present_2"
# subject = "calendar.calendar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.calendar: api_calendar_is_present_2 (surface)."""
import calendar

assert hasattr(calendar, "calendar")
print("api_calendar_is_present_2 OK")
