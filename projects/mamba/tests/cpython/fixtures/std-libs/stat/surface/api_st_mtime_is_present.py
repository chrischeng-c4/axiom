# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "api_st_mtime_is_present"
# subject = "stat.ST_MTIME"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stat.ST_MTIME: api_st_mtime_is_present (surface)."""
import stat

assert hasattr(stat, "ST_MTIME")
print("api_st_mtime_is_present OK")
