# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_thursday_is_present"
# subject = "calendar.THURSDAY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.THURSDAY: api_thursday_is_present (surface)."""
import calendar

assert hasattr(calendar, "THURSDAY")
print("api_thursday_is_present OK")
