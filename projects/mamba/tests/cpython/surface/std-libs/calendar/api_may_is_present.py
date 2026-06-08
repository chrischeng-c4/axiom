# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_may_is_present"
# subject = "calendar.MAY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.MAY: api_may_is_present (surface)."""
import calendar

assert hasattr(calendar, "MAY")
print("api_may_is_present OK")
