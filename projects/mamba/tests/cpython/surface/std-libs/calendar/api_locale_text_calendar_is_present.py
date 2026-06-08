# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_locale_text_calendar_is_present"
# subject = "calendar.LocaleTextCalendar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.LocaleTextCalendar: api_locale_text_calendar_is_present (surface)."""
import calendar

assert hasattr(calendar, "LocaleTextCalendar")
print("api_locale_text_calendar_is_present OK")
