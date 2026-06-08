# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "surface"
# case = "api_available_timezones_is_present"
# subject = "zoneinfo.available_timezones"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zoneinfo.available_timezones: api_available_timezones_is_present (surface)."""
import zoneinfo

assert hasattr(zoneinfo, "available_timezones")
print("api_available_timezones_is_present OK")
