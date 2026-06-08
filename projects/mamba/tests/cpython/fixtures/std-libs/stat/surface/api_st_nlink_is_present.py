# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "api_st_nlink_is_present"
# subject = "stat.ST_NLINK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stat.ST_NLINK: api_st_nlink_is_present (surface)."""
import stat

assert hasattr(stat, "ST_NLINK")
print("api_st_nlink_is_present OK")
