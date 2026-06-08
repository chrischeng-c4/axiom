# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_june_is_present"
# subject = "calendar.JUNE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.JUNE: api_june_is_present (surface)."""
import calendar

assert hasattr(calendar, "JUNE")
print("api_june_is_present OK")
