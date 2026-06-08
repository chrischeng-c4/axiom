# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_month_is_present"
# subject = "calendar.Month"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.Month: api_month_is_present (surface)."""
import calendar

assert hasattr(calendar, "Month")
print("api_month_is_present OK")
