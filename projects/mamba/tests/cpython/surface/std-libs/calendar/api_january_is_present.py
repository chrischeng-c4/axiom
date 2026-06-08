# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_january_is_present"
# subject = "calendar.JANUARY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.JANUARY: api_january_is_present (surface)."""
import calendar

assert hasattr(calendar, "JANUARY")
print("api_january_is_present OK")
