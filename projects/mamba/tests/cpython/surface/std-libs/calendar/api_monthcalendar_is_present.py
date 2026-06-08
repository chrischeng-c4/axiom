# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_monthcalendar_is_present"
# subject = "calendar.monthcalendar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.monthcalendar: api_monthcalendar_is_present (surface)."""
import calendar

assert hasattr(calendar, "monthcalendar")
print("api_monthcalendar_is_present OK")
