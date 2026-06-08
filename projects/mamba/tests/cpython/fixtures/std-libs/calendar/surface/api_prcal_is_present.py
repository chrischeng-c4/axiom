# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_prcal_is_present"
# subject = "calendar.prcal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.prcal: api_prcal_is_present (surface)."""
import calendar

assert hasattr(calendar, "prcal")
print("api_prcal_is_present OK")
