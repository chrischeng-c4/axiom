# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_locale_html_calendar_is_present"
# subject = "calendar.LocaleHTMLCalendar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.LocaleHTMLCalendar: api_locale_html_calendar_is_present (surface)."""
import calendar

assert hasattr(calendar, "LocaleHTMLCalendar")
print("api_locale_html_calendar_is_present OK")
