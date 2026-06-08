# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_august_is_present"
# subject = "calendar.AUGUST"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.AUGUST: api_august_is_present (surface)."""
import calendar

assert hasattr(calendar, "AUGUST")
print("api_august_is_present OK")
