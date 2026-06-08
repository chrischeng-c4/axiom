# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "api_st_mode_is_present"
# subject = "stat.ST_MODE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stat.ST_MODE: api_st_mode_is_present (surface)."""
import stat

assert hasattr(stat, "ST_MODE")
print("api_st_mode_is_present OK")
