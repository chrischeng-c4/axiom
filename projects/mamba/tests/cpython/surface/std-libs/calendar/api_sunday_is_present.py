# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_sunday_is_present"
# subject = "calendar.SUNDAY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.SUNDAY: api_sunday_is_present (surface)."""
import calendar

assert hasattr(calendar, "SUNDAY")
print("api_sunday_is_present OK")
