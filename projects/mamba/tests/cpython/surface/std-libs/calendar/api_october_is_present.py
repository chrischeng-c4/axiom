# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_october_is_present"
# subject = "calendar.OCTOBER"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.OCTOBER: api_october_is_present (surface)."""
import calendar

assert hasattr(calendar, "OCTOBER")
print("api_october_is_present OK")
