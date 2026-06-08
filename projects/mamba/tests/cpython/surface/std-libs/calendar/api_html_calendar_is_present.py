# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_html_calendar_is_present"
# subject = "calendar.HTMLCalendar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.HTMLCalendar: api_html_calendar_is_present (surface)."""
import calendar

assert hasattr(calendar, "HTMLCalendar")
print("api_html_calendar_is_present OK")
