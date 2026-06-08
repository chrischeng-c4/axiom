# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "filecmp"
# dimension = "surface"
# case = "api_clear_cache_is_present"
# subject = "filecmp.clear_cache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""filecmp.clear_cache: api_clear_cache_is_present (surface)."""
import filecmp

assert hasattr(filecmp, "clear_cache")
print("api_clear_cache_is_present OK")
