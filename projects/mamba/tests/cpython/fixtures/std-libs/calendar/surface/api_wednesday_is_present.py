# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_wednesday_is_present"
# subject = "calendar.WEDNESDAY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.WEDNESDAY: api_wednesday_is_present (surface)."""
import calendar

assert hasattr(calendar, "WEDNESDAY")
print("api_wednesday_is_present OK")
