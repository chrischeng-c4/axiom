# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_month_abbr_is_present"
# subject = "calendar.month_abbr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.month_abbr: api_month_abbr_is_present (surface)."""
import calendar

assert hasattr(calendar, "month_abbr")
print("api_month_abbr_is_present OK")
