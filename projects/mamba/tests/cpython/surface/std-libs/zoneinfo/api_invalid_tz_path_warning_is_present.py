# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "surface"
# case = "api_invalid_tz_path_warning_is_present"
# subject = "zoneinfo.InvalidTZPathWarning"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zoneinfo.InvalidTZPathWarning: api_invalid_tz_path_warning_is_present (surface)."""
import zoneinfo

assert hasattr(zoneinfo, "InvalidTZPathWarning")
print("api_invalid_tz_path_warning_is_present OK")
