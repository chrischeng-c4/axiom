# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_prmonth_is_present"
# subject = "calendar.prmonth"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.prmonth: api_prmonth_is_present (surface)."""
import calendar

assert hasattr(calendar, "prmonth")
print("api_prmonth_is_present OK")
