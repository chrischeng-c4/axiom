# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "surface"
# case = "api_zone_info_not_found_error_is_present"
# subject = "zoneinfo.ZoneInfoNotFoundError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zoneinfo.ZoneInfoNotFoundError: api_zone_info_not_found_error_is_present (surface)."""
import zoneinfo

assert hasattr(zoneinfo, "ZoneInfoNotFoundError")
print("api_zone_info_not_found_error_is_present OK")
