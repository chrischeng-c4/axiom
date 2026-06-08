# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "surface"
# case = "api_isleap_is_present"
# subject = "calendar.isleap"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""calendar.isleap: api_isleap_is_present (surface)."""
import calendar

assert hasattr(calendar, "isleap")
print("api_isleap_is_present OK")
