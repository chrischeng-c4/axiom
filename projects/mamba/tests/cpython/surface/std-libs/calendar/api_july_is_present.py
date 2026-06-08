# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_july_is_present"
# subject = "calendar.JULY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.JULY: api_july_is_present (surface)."""
import calendar

assert hasattr(calendar, "JULY")
print("api_july_is_present OK")
