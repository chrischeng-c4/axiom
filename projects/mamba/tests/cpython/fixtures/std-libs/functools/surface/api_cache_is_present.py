# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "api_cache_is_present"
# subject = "functools.cache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""functools.cache: api_cache_is_present (surface)."""
import functools

assert hasattr(functools, "cache")
print("api_cache_is_present OK")
