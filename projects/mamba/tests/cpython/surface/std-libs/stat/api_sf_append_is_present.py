# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "api_sf_append_is_present"
# subject = "stat.SF_APPEND"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stat.SF_APPEND: api_sf_append_is_present (surface)."""
import stat

assert hasattr(stat, "SF_APPEND")
print("api_sf_append_is_present OK")
