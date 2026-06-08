# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_timegm_is_present"
# subject = "calendar.timegm"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.timegm: api_timegm_is_present (surface)."""
import calendar

assert hasattr(calendar, "timegm")
print("api_timegm_is_present OK")
