# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_text_calendar_is_present"
# subject = "calendar.TextCalendar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.TextCalendar: api_text_calendar_is_present (surface)."""
import calendar

assert hasattr(calendar, "TextCalendar")
print("api_text_calendar_is_present OK")
