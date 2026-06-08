# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "api_sf_immutable_is_present"
# subject = "stat.SF_IMMUTABLE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stat.SF_IMMUTABLE: api_sf_immutable_is_present (surface)."""
import stat

assert hasattr(stat, "SF_IMMUTABLE")
print("api_sf_immutable_is_present OK")
