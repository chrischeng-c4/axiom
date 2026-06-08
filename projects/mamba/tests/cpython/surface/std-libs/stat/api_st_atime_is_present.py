# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "api_st_atime_is_present"
# subject = "stat.ST_ATIME"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stat.ST_ATIME: api_st_atime_is_present (surface)."""
import stat

assert hasattr(stat, "ST_ATIME")
print("api_st_atime_is_present OK")
