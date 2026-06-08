# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_march_is_present"
# subject = "calendar.MARCH"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.MARCH: api_march_is_present (surface)."""
import calendar

assert hasattr(calendar, "MARCH")
print("api_march_is_present OK")
