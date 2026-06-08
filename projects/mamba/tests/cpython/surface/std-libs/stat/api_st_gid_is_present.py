# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "api_st_gid_is_present"
# subject = "stat.ST_GID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stat.ST_GID: api_st_gid_is_present (surface)."""
import stat

assert hasattr(stat, "ST_GID")
print("api_st_gid_is_present OK")
