# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_september_is_present"
# subject = "calendar.SEPTEMBER"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.SEPTEMBER: api_september_is_present (surface)."""
import calendar

assert hasattr(calendar, "SEPTEMBER")
print("api_september_is_present OK")
