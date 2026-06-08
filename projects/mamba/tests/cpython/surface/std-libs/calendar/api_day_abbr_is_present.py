# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_day_abbr_is_present"
# subject = "calendar.day_abbr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.day_abbr: api_day_abbr_is_present (surface)."""
import calendar

assert hasattr(calendar, "day_abbr")
print("api_day_abbr_is_present OK")
