# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_saturday_is_present"
# subject = "calendar.SATURDAY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.SATURDAY: api_saturday_is_present (surface)."""
import calendar

assert hasattr(calendar, "SATURDAY")
print("api_saturday_is_present OK")
