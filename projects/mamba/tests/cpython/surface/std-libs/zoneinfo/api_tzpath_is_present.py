# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "surface"
# case = "api_tzpath_is_present"
# subject = "zoneinfo.TZPATH"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zoneinfo.TZPATH: api_tzpath_is_present (surface)."""
import zoneinfo

assert hasattr(zoneinfo, "TZPATH")
print("api_tzpath_is_present OK")
