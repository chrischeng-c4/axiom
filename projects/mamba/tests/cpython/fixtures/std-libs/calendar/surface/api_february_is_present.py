# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_february_is_present"
# subject = "calendar.FEBRUARY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.FEBRUARY: api_february_is_present (surface)."""
import calendar

assert hasattr(calendar, "FEBRUARY")
print("api_february_is_present OK")
