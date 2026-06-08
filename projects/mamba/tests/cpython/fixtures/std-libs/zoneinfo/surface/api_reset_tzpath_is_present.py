# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "surface"
# case = "api_reset_tzpath_is_present"
# subject = "zoneinfo.reset_tzpath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zoneinfo.reset_tzpath: api_reset_tzpath_is_present (surface)."""
import zoneinfo

assert hasattr(zoneinfo, "reset_tzpath")
print("api_reset_tzpath_is_present OK")
