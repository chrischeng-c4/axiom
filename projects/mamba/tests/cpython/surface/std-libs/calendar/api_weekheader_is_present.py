# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_weekheader_is_present"
# subject = "calendar.weekheader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.weekheader: api_weekheader_is_present (surface)."""
import calendar

assert hasattr(calendar, "weekheader")
print("api_weekheader_is_present OK")
